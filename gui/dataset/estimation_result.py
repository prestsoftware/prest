from __future__ import annotations

from dataclasses import dataclass
from fractions import Fraction
from typing import NamedTuple, Sequence, List, Iterator, \
    Optional, Any, NewType, cast, Callable

from PyQt5.QtGui import QIcon
from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import QDialog, QHeaderView

import gui
import model
import dataset
import platform_specific
import dataset.aggregated_preferences
from dataset.aggregated_preferences import InstanceRepr, InstanceReprC, \
    display_instance, instance_repr_to_code, \
    PackedEstimationResponse, PackedEstimationResponseC, \
    PackedEstimationResponsesC  # noqa: F401 (used as public reexport)
from core import Core
import gui.aggregate
from gui.progress import Worker
from gui.estimation import DistanceScore, distanceScoreC
from model import get_name as model_get_name
from model import Model as ModelRepr
from model import ModelC
from model import get_ordering_key as model_get_ordering_key
from dataset import Dataset, DatasetHeaderC, ExportVariant, Analysis
from util.tree_model import Node, TreeModel, Field, PackedRootNode
from util.codec import Codec, FileIn, FileOut, namedtupleC, strC, intC, \
    listC, bytesC, tupleC, boolC, fractionC
from util.codec_progress import CodecProgress, listCP, oneCP
import uic.view_estimated

def from_fraction(x : Fraction) -> int | float:
    if x.denominator == 1:
        return x.numerator
    else:
        return round(float(x), 3)  # sounds about appropriate

class Penalty(NamedTuple):
    # both bounds are inclusive
    lower_bound: Fraction
    upper_bound: Fraction

    def __str__(self):
        return str(self.to_csv())

    def to_csv(self) -> str | int | float:
        if self.lower_bound != self.upper_bound:
            return f'{self.lower_bound-1} < N ≤ {self.upper_bound}'
        else:
            return from_fraction(self.upper_bound)

PenaltyC = namedtupleC(Penalty, fractionC, fractionC)

class Request(NamedTuple):
    subjects : list[dataset.PackedSubject]
    models : Sequence[model.Model]
    disable_parallelism : bool
    disregard_deferrals : bool
    distance_score : DistanceScore

RequestC = namedtupleC(Request, listC(dataset.PackedSubjectC), listC(ModelC), boolC, boolC, distanceScoreC)

class InstanceInfo(NamedTuple):
    model : ModelRepr
    penalty : Penalty
    instance : InstanceRepr

InstanceInfoC = namedtupleC(InstanceInfo, ModelC, PenaltyC, InstanceReprC)

class Response(NamedTuple):
    subject_name : str
    penalty : Penalty
    best_instances : list[InstanceInfo]

ResponseC = namedtupleC(Response, strC, PenaltyC, listC(InstanceInfoC))
ResponsesC = listC(ResponseC)

class Instance(NamedTuple):
    model: str
    data: InstanceRepr

InstanceC = namedtupleC(Instance, strC, InstanceReprC)

class Subject(NamedTuple):
    name: str
    penalty: Penalty
    best_models: list[tuple[model.Model, Penalty, list[InstanceRepr]]]

SubjectC = namedtupleC(Subject, strC, PenaltyC, listC(tupleC(ModelC, PenaltyC, listC(InstanceReprC))))

PackedSubject = NewType('PackedSubject', bytes)
PackedSubjectC = cast(Codec[PackedSubject], bytesC)

def subject_from_response_bytes(response_bytes : PackedEstimationResponse) -> Subject:
    # returns something orderable
    def model_sort_criterion(chunk : tuple[ModelRepr, tuple[Penalty, list[InstanceRepr]]]) -> Any:
        model, (penalty, instances) = chunk
        return (
            model_get_ordering_key(model),
            len(instances),
            len(model_get_name(model)),
        )

    subject_name, subject_penalty, best_instances = ResponseC.decode_from_memory(response_bytes)

    by_model: dict[model.Model, tuple[Penalty, list[InstanceRepr]]] = {}
    for this_model, inst_penalty, instance in best_instances:
        penalty_instances_so_far = by_model.get(this_model)
        if penalty_instances_so_far:
            penalty_so_far, instances_so_far = penalty_instances_so_far
            assert penalty_so_far == inst_penalty, f'assertion failed: {inst_penalty} ≠ {penalty_so_far}'
            instances_so_far.append(instance)
        else:
            by_model[this_model] = (inst_penalty, [instance])

    return Subject(
        name=subject_name,
        penalty=subject_penalty,
        best_models=[
            (model, penalty, instances)
            for model, (penalty, instances)
            in sorted(by_model.items(), key=model_sort_criterion)
        ],
    )

@dataclass
class ChainStats:
    count : int
    length : int

class EstimationResult(Dataset):
    class Subject(Node):
        def __init__(self, parent_node, row: int, subject: Subject) -> None:
            Node.__init__(
                self, parent_node, row,
                fields=(subject.name, str(subject.penalty), '%d models' % len(subject.best_models)),
                child_count=len(subject.best_models),
            )
            self.subject = subject

        def create_child(self, row: int) -> 'EstimationResult.Model':
            model, penalty, instances = self.subject.best_models[row]
            return EstimationResult.Model(self, row, model, penalty, instances)

    class Model(Node):
        def __init__(self, parent_node: 'EstimationResult.Subject', row: int,
            model: model.Model, penalty: Penalty, instances: list[InstanceRepr]
        ) -> None:
            subject = parent_node.subject
            Node.__init__(
                self, parent_node, row,
                fields=(model_get_name(model), penalty, '%d instances' % len(instances)),
                child_count=len(instances),
            )
            self.instances = instances
            self.subject = subject

        def create_child(self, row: int) -> 'EstimationResult.Instance':
            return EstimationResult.Instance(self, row, self.instances[row])

    class Instance(Node):
        def __init__(self, parent_node: 'EstimationResult.Model', row: int, instance: InstanceRepr) -> None:
            code = instance_repr_to_code(instance)
            #subject = parent_node.subject

            help_icon = QIcon(platform_specific.get_embedded_file_path('images/qm-16.png'))
            Node.__init__(
                self, parent_node, row,
                fields=(code, Field(icon=help_icon, user_data=code), ''),
                # fields=(code, '', ''),
            )

    class ViewDialog(uic.view_estimated.Ui_ViewEstimated, gui.ExceptionDialog):
        def __init__(self, ds: 'EstimationResult') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.alternatives = ds.alternatives
            self.model = TreeModel(
                PackedRootNode(
                    EstimationResult.Subject,
                    cast(Callable[[bytes], Any], subject_from_response_bytes),
                    'Subject',
                    ds.subjects
                ),
                headers=('Name', 'Distance score', 'Size'),
            )
            self.twSubjects.setModel(self.model)
            self.twSubjects.header().setSectionResizeMode(QHeaderView.ResizeToContents)
            self.twSubjects.header().setStretchLastSection(False)

            self.twSubjects.clicked.connect(self.catch_exc(self.dlg_item_clicked))

        def dlg_item_clicked(self, idx):
            instance_code = cast(str, self.model.data(idx, Qt.UserRole))
            if instance_code:
                display_instance(self.alternatives, instance_code)

    def __init__(self, name: str, alternatives: Sequence[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: List[PackedEstimationResponse] = []

    def get_stats(self) -> ChainStats:
        n_chains = 1
        for subj_packed in self.subjects:
            subj = subject_from_response_bytes(subj_packed)
            for _model, _penalty, instances in subj.best_models:
                n_chains *= len(instances)

        return ChainStats(
            count=n_chains,
            length=len(self.subjects),
        )

    def config_aggregate_preferences(self, _experimental_features : bool) -> Optional[gui.aggregate.Mode]:
        chains = self.get_stats()
        dlg = gui.aggregate.ConfigAggregated(
            chain_count=chains.count,
            chain_length=chains.length,
        )
        if dlg.exec() == QDialog.Accepted:
            return dlg.value()
        else:
            return None

    def analysis_aggregate_preferences(self, worker : Worker, config : gui.aggregate.Mode) -> dataset.AnalysisResult:
        worker.set_work_size(1)

        with Core() as core:
            worker.interrupt = lambda: core.shutdown()

            response = core.call(
                'aggregate-preferences',
                dataset.aggregated_preferences.RequestC,
                dataset.aggregated_preferences.ResponseC,
                dataset.aggregated_preferences.Request(
                    mode=config,
                    subjects=self.subjects,
                )
            )

            worker.set_progress(1)

        ds = dataset.aggregated_preferences.AggregatedPreferences(
            self.name + ' (aggregated)',
            self.alternatives,
            response,
        )
        return ds

    def get_analyses(self) -> Sequence[Analysis]:
        return (
            Analysis(
                name='Aggregate preferences',
                config=self.config_aggregate_preferences,
                run=self.analysis_aggregate_preferences,
            ),
        )

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Compact (human-friendly)',
                column_names=('subject', 'dist_score', 'model', 'instances'),
                get_rows=self.export_compact,
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Detailed (machine-friendly)',
                column_names=('subject', 'dist_score', 'dist_score_upper_bound', 'model', 'instance'),
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        )

    def export_detailed(self) -> Iterator[Optional[tuple[str,Optional[int|float],int|float,str,str]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            for model, penalty, instances in subject.best_models:
                for instance in sorted(instances):
                    yield (
                        subject.name,
                        from_fraction(penalty.lower_bound)
                            if penalty.lower_bound == penalty.upper_bound
                            else None,
                        from_fraction(penalty.upper_bound),
                        model_get_name(model),
                        instance_repr_to_code(instance),
                    )

            yield None  # bump progress

    def export_compact(self) -> Iterator[Optional[tuple[Optional[str],str|int|float,str,int]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            subject_name: Optional[str] = subject.name
            for model, model_penalty, instances in subject.best_models:
                yield (subject_name, model_penalty.to_csv(), model_get_name(model), len(instances))
                subject_name = None  # don't repeat these

            yield None  # bump progress

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress[EstimationResult]:
        subjects_encode : Callable[[Worker, FileOut, list[PackedEstimationResponse]], None]
        subjects_decode : Callable[[Worker, FileIn], list[PackedEstimationResponse]]

        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(PackedEstimationResponseC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : EstimationResult) -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : EstimationResult) -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> EstimationResult:
            ds = EstimationResult(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
