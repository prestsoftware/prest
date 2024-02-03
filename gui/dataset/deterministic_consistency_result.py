from __future__ import annotations

import logging
from typing import NamedTuple, List, Iterator, Tuple, \
    Sequence, Optional, Union

from PyQt5.QtGui import QIcon
from PyQt5.QtWidgets import QDialog, QHeaderView

import doc
import uic.view_dataset
import util.tree_model
import platform_specific
from gui.progress import Worker
from dataset import Dataset, DatasetHeaderC, ExportVariant, Analysis
from util.codec import FileIn, FileOut, namedtupleC, strC, intC, listC
from util.codec_progress import CodecProgress, listCP, oneCP
from util.tree_model import Node

log = logging.getLogger(__name__)

class Row(NamedTuple):
    cycle_length: int
    garp: int
    sarp: int
    garp_binary_menus: int
    sarp_binary_menus: int
    binary_intransitivities : int

RowC = namedtupleC(Row, intC, intC, intC, intC, intC, intC)

class SubjectRaw(NamedTuple):
    name: str
    rows: List[Row]
    warp_pairs: int
    warp_all: int
    contraction_consistency_pairs : int
    contraction_consistency_all : int

SubjectRawC = namedtupleC(SubjectRaw, strC, listC(RowC), intC, intC, intC, intC)

class Subject(NamedTuple):
    raw: SubjectRaw

    total_garp: int
    total_sarp: int
    total_garp_binary_menus: int
    total_sarp_binary_menus: int
    total_binary_intransitivities: int

    @staticmethod
    def from_raw(raw : SubjectRaw) -> 'Subject':
        return Subject(
            raw,
            total_garp=sum(r.garp for r in raw.rows),
            total_sarp=sum(r.sarp for r in raw.rows),
            total_garp_binary_menus=sum(r.garp_binary_menus for r in raw.rows),
            total_sarp_binary_menus=sum(r.sarp_binary_menus for r in raw.rows),
            total_binary_intransitivities=sum(r.binary_intransitivities for r in raw.rows),
        )

SubjectC = namedtupleC(Subject, SubjectRawC, intC, intC, intC, intC, intC)

class RootNode(util.tree_model.RootNode):
    def __init__(self, subjects: List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects

    def create_child(self, row: int) -> 'SubjectNode':
        return SubjectNode(self, row, self.subjects[row])

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node : Node, row: int, subject: Subject) -> None:
        self.subject = subject

        if len(subject.raw.rows) == 0:
            util.tree_model.Node.__init__(
                self, parent_node, row,
                fields=(
                    subject.raw.name,
                    '',
                    subject.raw.contraction_consistency_pairs,
                    subject.raw.contraction_consistency_all,
                    subject.raw.warp_pairs,
                    subject.raw.warp_all,
                    0, 0, 0, 0, 0),
            )
        elif len(subject.raw.rows) == 1:
            util.tree_model.Node.__init__(
                self, parent_node, row,
                fields=(
                    subject.raw.name,
                    subject.raw.rows[0].cycle_length,
                    subject.raw.contraction_consistency_pairs,
                    subject.raw.contraction_consistency_all,
                    subject.raw.warp_pairs,
                    subject.raw.warp_all,

                    subject.total_garp,
                    subject.total_sarp,
                    subject.total_garp_binary_menus,
                    subject.total_sarp_binary_menus,
                    subject.total_binary_intransitivities,
                )
            )
        else:
            util.tree_model.Node.__init__(
                self, parent_node, row,
                fields=(
                    subject.raw.name,
                    '',
                    subject.raw.contraction_consistency_pairs,
                    subject.raw.contraction_consistency_all,
                    subject.raw.warp_pairs,
                    subject.raw.warp_all,

                    subject.total_garp,
                    subject.total_sarp,
                    subject.total_garp_binary_menus,
                    subject.total_sarp_binary_menus,
                    subject.total_binary_intransitivities,
                ),
                child_count = len(subject.raw.rows),
            )

    def create_child(self, row: int) -> 'RowNode':
        return RowNode(self, row, self.subject.raw.rows[row])

class RowNode(util.tree_model.Node):
    def __init__(self, parent_node : Node, row_no: int, row: Row) -> None:
        util.tree_model.Node.__init__(
            self, parent_node, row_no,
            fields=(
                '',
                row.cycle_length,
                '-',
                '-',
                '-',
                '-',
                row.garp,
                row.sarp,
                row.garp_binary_menus,
                row.sarp_binary_menus,
                row.binary_intransitivities,
            )
        )

Cycles = List[Tuple[int, int]]

class DeterministicConsistencyResult(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds: DeterministicConsistencyResult) -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            help_icon = QIcon(platform_specific.get_embedded_file_path(
                'images/qm-16.png',      # deploy
                'gui/images/qm-16.png',  # devel
            ))

            # we assign model to self to prevent GC
            F = util.tree_model.Field
            self.model = util.tree_model.TreeModel(
                RootNode(ds.subjects),
                headers=(
                    'Subject',
                    'Cycle length',
                    'Contraction consistency (pairs)',
                    'Contraction consistency (all)',
                    F('WARP (pairs)', help_icon,
                        'consistency/cons_general.html#weak-axiom-of-revealed-preference-warp'),
                    F('WARP (all)', help_icon,
                        'consistency/cons_general.html#weak-axiom-of-revealed-preference-warp'),
                    F('Congruence', help_icon,
                        'consistency/cons_general.html#congruence'),
                    F('Strict general cycles', help_icon,
                        'consistency/cons_general.html#strict-choice-consistency'),  # SARP
                    F('Binary cycles', help_icon,
                        'consistency/cons_general.html#binary-choice-consistency'),
                    F('Strict binary cycles', help_icon,
                        'consistency/cons_general.html#strict-binary-choice-consistency'),  # SARP-binary
                    'Binary intransitivities',
                ),
            )
            self.twRows.setModel(self.model)

            hdr = self.twRows.header()

            hdr.setSectionsClickable(True)
            hdr.sectionClicked.connect(self.header_clicked)

            hdr.setSectionResizeMode(QHeaderView.ResizeToContents)
            hdr.setStretchLastSection(False)

        def header_clicked(self, idx : int) -> None:
            name = self.model.headers[idx].user_data
            if not name:
                return

            doc.open_in_browser(name)

    def __init__(self, name: str, alternatives: List[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: List[Subject] = []
        self.max_cycle_length : int = 0

    def load_from_core(self, raws: List[SubjectRaw]) -> None:
        self.subjects = [Subject.from_raw(raw) for raw in raws]
        self.max_cycle_length = max(
            (row.cycle_length for subj in self.subjects for row in subj.raw.rows),
            default=0
        )

    def get_analyses(self) -> Sequence[Analysis]:
        return []

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Summary',
                column_names=(
                    'subject',
                    'contraction_consistency_pairs',
                    'contraction_consistency_all',
                    'warp_pairs',
                    'warp_all',
                    'congruence',
                    'strict_general_cycles',
                    'binary_cycles',
                    'strict_binary_cycles',
                    'binary_intransitivities',
                ),
                get_rows=self.export_summary,
                size=len(self.subjects),
            ),
            ExportVariant(
                name='WARP violations',
                column_names=['subject', 'warp_pairs', 'warp_all'],
                get_rows=self.export_warp,
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Congruence violations (wide)',
                column_names=['subject'] + ['cycles_%d' % l for l in range(3,self.max_cycle_length+1)] + ['total'],
                get_rows=lambda: self.export_wide('garp'),
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Strict general cycles (wide)',
                column_names=['subject'] + ['cycles_%d' % l for l in range(3,self.max_cycle_length+1)] + ['total'],
                get_rows=lambda: self.export_wide('sarp'),
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Strict binary cycles (wide)',
                column_names=['subject'] + ['cycles_%d' % l for l in range(3,self.max_cycle_length+1)] + ['total'],
                get_rows=lambda: self.export_wide('sarp_binary_menus'),
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Binary cycles (wide)',
                column_names=['subject'] + ['cycles_%d' % l for l in range(3,self.max_cycle_length+1)] + ['total'],
                get_rows=lambda: self.export_wide('garp_binary_menus'),
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Binary intransitivities (wide)',
                column_names=['subject'] + ['cycles_%d' % l for l in range(3,self.max_cycle_length+1)] + ['total'],
                get_rows=lambda: self.export_wide('binary_intransitivities'),
                size=len(self.subjects),
            ),
        )

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    def export_warp(self) -> Iterator[Optional[Tuple[str, int, int]]]:
        for subject in self.subjects:
            yield (subject.raw.name, subject.raw.warp_pairs, subject.raw.warp_all)
            yield None  # bump progress

    def export_wide(self, column_name : str) -> Iterator[Optional[Tuple[Union[int, str], ...]]]:
        index = Row._fields.index(column_name)

        for subject in self.subjects:
            count_by_length = [0 for _ in range(3, self.max_cycle_length+1)]
            for row in subject.raw.rows:
                count_by_length[row.cycle_length-3] = row[index]

            yield (subject.raw.name, *count_by_length, sum(count_by_length))
            yield None  # bump progress

    def export_summary(self) -> Iterator[Optional[
        Tuple[str,int,int,int,int,int,int,int,int,int]
    ]]:
        for subject in self.subjects:
            yield (
                subject.raw.name,
                subject.raw.contraction_consistency_pairs,
                subject.raw.contraction_consistency_all,
                subject.raw.warp_pairs,
                subject.raw.warp_all,
                sum(row.garp for row in subject.raw.rows),
                sum(row.sarp for row in subject.raw.rows),
                sum(row.garp_binary_menus for row in subject.raw.rows),
                sum(row.sarp_binary_menus for row in subject.raw.rows),
                subject.total_binary_intransitivities,
            )

            yield None

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress[DeterministicConsistencyResult]:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : DeterministicConsistencyResult) -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : DeterministicConsistencyResult) -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)
            intC_encode(f, x.max_cycle_length)

        def decode(worker : Worker, f : FileIn) -> DeterministicConsistencyResult:
            ds = DeterministicConsistencyResult(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            ds.max_cycle_length = intC_decode(f)
            return ds

        return CodecProgress(get_size, encode, decode)
