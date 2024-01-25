from __future__ import annotations

import logging
import collections
from dataclasses import dataclass
from typing import Sequence, Iterator, NamedTuple, Optional

from PyQt5.QtWidgets import QDialog, QHeaderView

import dataset
import gui.copycat_simulation
import gui.estimation
import simulation
from core import Core
from dataset import Dataset, DatasetHeaderC, ChoiceRow, \
    Subject, SubjectC, ExportVariant, Analysis, PackedSubject, PackedSubjectC
from gui.progress import Worker
from dataset.estimation_result import EstimationResult
from dataset.stochastic_consistency_result import StochasticConsistencyResult
from dataset.deterministic_consistency_result import DeterministicConsistencyResult
from dataset.experiment_stats import ExperimentStats
from dataset.tuple_intrans_alts import TupleIntransAlts
from dataset.tuple_intrans_menus import TupleIntransMenus
import dataset.deterministic_consistency_result
import dataset.experiment_stats
import dataset.tuple_intrans_alts
import dataset.tuple_intrans_menus
import dataset.integrity_check
import dataset.estimation_result as estimation_result
import uic.view_dataset
import util.tree_model
from util.codec import FileIn, FileOut, namedtupleC, strC, intC, \
    frozensetC, maybe
from util.codec_progress import CodecProgress, listCP, oneCP

log = logging.getLogger(__name__)

class ChoiceRow_str(NamedTuple):
    menu : frozenset[str]
    default : Optional[str]
    choice : frozenset[str]

ChoiceRow_strC = namedtupleC(ChoiceRow_str, frozensetC(strC), maybe(strC), frozensetC(strC))

class ChoiceRowNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, cr: ChoiceRow) -> None:
        subject = parent_node.subject
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(subject.name, subject.csv_set(cr.menu), subject.csv_alt(cr.default), subject.csv_set(cr.choice)),
        )

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, subject : Subject) -> None:
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(subject.name, None, None),
            child_count=len(subject.choices),
        )
        self.subject = subject

    def create_child(self, row: int) -> ChoiceRowNode:
        return ChoiceRowNode(self, row, self.subject.choices[row])

def parse_set(s: str) -> frozenset[str]:
    s = s.strip()

    if s == '':
        return frozenset()

    return frozenset(alt.strip() for alt in s.split(','))

class CsvError(Exception):
    pass

class ExperimentalData(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds: ExperimentalData) -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            # we assign model to self to prevent GC
            self.model = util.tree_model.TreeModel(
                util.tree_model.PackedRootNode(SubjectNode, SubjectC.decode_from_memory, 'Subject', ds.subjects),
                headers=('Subject', 'Menu', 'Default', 'Choice'),
            )
            self.twRows.setModel(self.model)

            self.twRows.header().setSectionResizeMode(QHeaderView.ResizeToContents)
            self.twRows.header().setStretchLastSection(False)

    def __init__(self, name: str, alternatives: Sequence[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: list[PackedSubject] = []
        self.observ_count: int = 0

    @staticmethod
    def from_csv(name: str, rows: Sequence[Sequence[str]], indices: tuple[int,int,Optional[int],int]) -> ExperimentalData:
        i_s, i_m, i_d, i_c = indices  # CSV column indices: subject, menu, default, choice

        subjects_raw: dict[str,list[ChoiceRow_str]] = collections.defaultdict(list)
        for row in rows:
            # row[i_s] is the name of the subject
            cr = ChoiceRow_str(
                menu=parse_set(row[i_m]),
                default=(row[i_d] if i_d is not None else None) or None,  # empty string -> None
                choice=parse_set(row[i_c]),
            )

            if (cr.default is not None) and (cr.default not in cr.menu):
                raise CsvError('%s: default alternative "%s" does not appear in its menu "%s".' % (
                    row[i_s], cr.default, set(cr.menu)
                ))

            subjects_raw[row[i_s]].append(cr)

        subjects: list[PackedSubject] = []
        alternatives_dataset: set[str] = set()
        observ_count = 0

        for subject_name, choices in subjects_raw.items():
            alternatives_subj: set[str] = set()
            for cr in choices:
                alternatives_subj |= cr.menu
                alternatives_subj |= cr.choice

            alternatives_dataset |= alternatives_subj

            alternatives = sorted(alternatives_subj)  # order matters!
            alt_map = dict()
            for i, alt in enumerate(alternatives):
                alt_map[alt] = i

            subject = Subject(
                name=subject_name,
                alternatives=alternatives,
                choices=[
                    ChoiceRow(
                        menu=frozenset(alt_map[x] for x in cr.menu),
                        default=alt_map[cr.default] if cr.default else None,  # cr.default == "" -> None
                        choice=frozenset(alt_map[x] for x in cr.choice),
                    )
                    for cr in choices
                ]
            )
            observ_count += len(subject.choices)
            subjects.append(PackedSubject(SubjectC.encode_to_memory(subject)))

        ds = ExperimentalData(name, sorted(alternatives_dataset))
        ds.subjects = subjects
        ds.observ_count = observ_count
        return ds

    def clear_subjects(self):
        self.subjects = []
        self.observ_count = 0

    def label_size(self):
        return '%d subjs, %d observations' % (len(self.subjects), self.observ_count)

    def config_estimation(self, _experimental_features : bool) -> Optional[gui.estimation.Options]:
        dlg = gui.estimation.Estimation()
        if dlg.exec() == QDialog.Accepted:
            return dlg.value()
        else:
            return None

    def config_simulation(self, experimental_features : bool) -> Optional[gui.copycat_simulation.Options]:
        dlg = gui.copycat_simulation.CopycatSimulation(self, experimental_features)
        if dlg.exec() == QDialog.Accepted:
            return dlg.value()
        else:
            return None

    def analysis_simulation(self, worker : Worker, options : 'gui.copycat_simulation.Options') -> ExperimentalData:
        subjects : list[PackedSubject] = []

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()  # register interrupt hook

            worker.set_work_size(len(self.subjects) * options.multiplicity)
            position = 0
            iteration_counter = 0
            for subject_packed in self.subjects:
                for j in range(options.multiplicity):
                    while True:
                        iteration_counter += 1
                        response = simulation.run(core, simulation.Request(
                            name='random%d' % (j+1),
                            alternatives=self.alternatives,  # we don't use subject.alternatives here
                            gen_menus=simulation.GenMenus(
                                generator=simulation.Copycat(subject_packed),
                                defaults=False,  # this will be ignored, anyway
                            ),
                            gen_choices=options.gen_choices,
                            preserve_deferrals=options.preserve_deferrals,
                        ))

                        subject_accepted = gui.subject_filter.accepts(
                            options.subject_filter,
                            core,
                            response.subject_packed,
                        )

                        if subject_accepted:
                            subjects.append(response.subject_packed)

                            position += 1
                            if iteration_counter % 256 == 0:
                                worker.set_progress(position)
                            break
                        else:
                            continue

        ds = ExperimentalData(name=options.name, alternatives=self.alternatives)
        ds.subjects = subjects
        ds.observ_count = options.multiplicity * self.observ_count
        return ds

    @dataclass
    class MergeOptions:
        track_deferrals_separately : bool

    def config_merge_choices(self, _experimental_features : bool) -> Optional[MergeOptions]:
        # hardwire this without UI for now
        return ExperimentalData.MergeOptions(
            track_deferrals_separately=False
        )

    def analysis_merge_choices(self, worker : Worker, config : MergeOptions) -> ExperimentalData:
        subjects : list[PackedSubject] = []
        observ_count : int = 0

        # we group by pairs (menu, default)
        MenuDef = tuple[frozenset[int], Optional[int]]

        worker.set_work_size(len(self.subjects))
        for i, subject_packed in enumerate(self.subjects):
            subject = Subject.unpack(subject_packed)

            choices : list[ChoiceRow] = []
            menu_idx : dict[MenuDef, int] = {}
            deferrals_seen : set[MenuDef] = set()

            for cr in subject.choices:
                # deferrals are kept separately if requested
                if config.track_deferrals_separately and (not cr.choice):
                    if (cr.menu, cr.default) in deferrals_seen:
                        # this deferral has already been seen, just skip it
                        continue
                    else:
                        # this is the first time we've seen deferral at this menu
                        # add that deferral to the output
                        # but don't add it to the index
                        choices.append(cr)
                        deferrals_seen.add((cr.menu, cr.default))
                        observ_count += 1
                        continue

                idx = menu_idx.get((cr.menu, cr.default))
                if idx is None:
                    # not there yet
                    menu_idx[cr.menu, cr.default] = len(choices)
                    choices.append(cr)
                    observ_count += 1
                else:
                    # already there, expand the choice
                    choices[idx] = ChoiceRow(
                        menu=cr.menu,
                        default=cr.default,
                        choice=cr.choice | choices[idx].choice,
                    )

            subjects.append(Subject(
                name=subject.name,
                alternatives=subject.alternatives,
                choices=choices,
            ).pack())
            worker.set_progress(i+1)

        ds = ExperimentalData(name=self.name + ' (merged)', alternatives=self.alternatives)
        ds.subjects = subjects
        ds.observ_count = observ_count
        return ds

    def analysis_estimation(self, worker : Worker, options : gui.estimation.Options) -> EstimationResult:
        CHUNK_SIZE = 64
        with Core() as core:
            worker.interrupt = lambda: core.shutdown()  # register interrupt hook

            rows : list[estimation_result.PackedResponse] = []
            worker.set_work_size(len(self.subjects))
            for i in range(0, len(self.subjects), CHUNK_SIZE):
                request = estimation_result.Request(
                    subjects=self.subjects[i:i+CHUNK_SIZE],
                    models=options.models,
                    disable_parallelism=options.disable_parallelism,
                    disregard_deferrals=options.disregard_deferrals,
                    distance_score=options.distance_score,
                )

                responses = core.call(
                    'estimation',
                    estimation_result.RequestC,
                    estimation_result.PackedResponsesC,
                    request
                )
                rows.extend(responses)

                worker.set_progress(len(rows))

            if options.distance_score != gui.estimation.DistanceScore.HOUTMAN_MAKS:
                suffix = f' (model est., {options.distance_score.value})'
            else:
                suffix = ' (model est.)'

            ds = EstimationResult(
                self.name + suffix,
                self.alternatives,
            )
            ds.subjects = rows

        return ds

    def analysis_consistency_deterministic(self, worker : Worker, _config : None) -> DeterministicConsistencyResult:
        with Core() as core:
            worker.interrupt = lambda: core.shutdown()  # interrupt hook

            rows = []

            worker.set_work_size(len(self.subjects))
            for i, subject in enumerate(self.subjects):
                response = core.call(
                    'consistency-deterministic',
                    PackedSubjectC,
                    dataset.deterministic_consistency_result.SubjectRawC,
                    subject
                )
                rows.append(response)

                worker.set_progress(i+1)

        ds = DeterministicConsistencyResult(
            self.name + ' (deterministic consistency)',
            self.alternatives,
        )
        ds.load_from_core(rows)
        return ds

    def analysis_consistency_stochastic(self, worker : Worker, _config : None) -> StochasticConsistencyResult:
        with Core() as core:
            worker.interrupt = lambda: core.shutdown()  # interrupt hook

            rows = []

            worker.set_work_size(len(self.subjects))
            for i, subject in enumerate(self.subjects):
                response = core.call(
                    'consistency-stochastic',
                    PackedSubjectC,
                    dataset.stochastic_consistency_result.SubjectC,
                    subject
                )
                rows.append(response)

                worker.set_progress(i+1)

        ds = StochasticConsistencyResult(
            self.name + ' (stochastic consistency)',
            self.alternatives,
        )
        ds.subjects = rows
        return ds

    def analysis_summary_stats(self, worker : Worker, _config : None) -> ExperimentStats:
        subjects = []
        worker.set_work_size(len(self.subjects))

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()

            for i, subject in enumerate(self.subjects):
                subjects.append(core.call(
                    "summary",
                    PackedSubjectC,
                    dataset.experiment_stats.SubjectC,
                    subject
                ))
                worker.set_progress(i+1)

        ds = ExperimentStats(
            name=self.name + ' (info)',
            alternatives=self.alternatives,
        )
        ds.subjects = subjects
        return ds

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Detailed',
                column_names=('subject', 'menu', 'default', 'choice'),
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        )

    def export_detailed(self) -> Iterator[Optional[tuple[str,str,Optional[str],str]]]:
        for subject in map(SubjectC.decode_from_memory, self.subjects):
            for cr in subject.choices:
                yield (
                    subject.name,
                    subject.csv_set(cr.menu),
                    subject.csv_alt(cr.default),
                    subject.csv_set(cr.choice),
                )

            yield None  # bump progress

    def analysis_tuple_intrans_menus(self, worker : Worker, _config : None) -> TupleIntransMenus:
        subjects = []
        worker.set_work_size(len(self.subjects))

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()

            for i, subject in enumerate(self.subjects):
                subjects.append(
                    core.call(
                        'tuple-intrans-menus',
                        PackedSubjectC,
                        dataset.tuple_intrans_menus.SubjectC,
                        subject,
                    )
                )
                worker.set_progress(i+1)

        ds = TupleIntransMenus(self.name + ' (inconsistent menu tuples)', self.alternatives)
        ds.subjects = subjects
        return ds

    def analysis_tuple_intrans_alts(self, worker : Worker, _config : None) -> TupleIntransAlts:
        subjects = []
        worker.set_work_size(len(self.subjects))

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()

            for i, subject in enumerate(self.subjects):
                subjects.append(
                    core.call(
                        'tuple-intrans-alts',
                        PackedSubjectC,
                        dataset.tuple_intrans_alts.SubjectC,
                        subject,
                    )
                )
                worker.set_progress(i+1)

        ds = TupleIntransAlts(self.name + ' (inconsistent alternative tuples)', self.alternatives)
        ds.subjects = subjects
        return ds

    def analysis_integrity_check(self, worker : Worker, _config : None) -> dataset.AnalysisResult:
        worker.set_work_size(len(self.subjects))

        subjects : list[dataset.integrity_check.Subject] = []

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()

            for i, subject in enumerate(self.subjects):
                subj_issues = core.call(
                    'integrity-check',
                    PackedSubjectC,
                    dataset.integrity_check.SubjectC,
                    subject
                )

                if subj_issues.issues:
                    subjects.append(subj_issues)

                worker.set_progress(i+1)

        if subjects:
            ds = dataset.integrity_check.IntegrityCheck(self.name + ' (integrity check)', self.alternatives)
            ds.subjects = subjects
            return ds
        else:
            return dataset.ShowMessageBox(
                type=dataset.MessageBoxType.INFORMATION,
                title='Integrity check',
                message='No integrity issues found.',
            )

    def get_analyses(self) -> Sequence[Analysis]:
        return (
            Analysis(
                name='Integrity check',
                config=None,
                run=self.analysis_integrity_check,
            ),
            Analysis(
                name='Summary information',
                config=None,
                run=self.analysis_summary_stats,
            ),
            Analysis(
                name='Deterministic consistency analysis',
                config=None,
                run=self.analysis_consistency_deterministic,
            ),
            Analysis(
                name='Stochastic consistency analysis',
                config=None,
                run=self.analysis_consistency_stochastic,
            ),
            Analysis(
                name='Inconsistent tuples of menus',
                config=None,
                run=self.analysis_tuple_intrans_menus,
            ),
            Analysis(
                name='Inconsistent tuples of alternatives',
                config=None,
                run=self.analysis_tuple_intrans_alts,
            ),
            Analysis(
                name='Model estimation',
                config=self.config_estimation,
                run=self.analysis_estimation,
            ),
            Analysis(
                name='Merge choices at the same menu',
                config=self.config_merge_choices,
                run=self.analysis_merge_choices,
            ),
            Analysis(
                name='Generate similar random dataset',
                config=self.config_simulation,
                run=self.analysis_simulation,
            ),
        )

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress[ExperimentalData]:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(PackedSubjectC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : ExperimentalData) -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : ExperimentalData) -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)
            intC_encode(f, x.observ_count)

        def decode(worker : Worker, f : FileIn) -> ExperimentalData:
            ds = ExperimentalData(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            ds.observ_count = intC_decode(f)
            return ds

        return CodecProgress(get_size, encode, decode)
