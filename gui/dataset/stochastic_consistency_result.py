from __future__ import annotations

import logging
from typing import Iterator, Sequence, Optional

from PyQt5.QtWidgets import QDialog, QHeaderView

import doc
import uic.view_dataset
import util.tree_model
from gui.progress import Worker
from dataset import Dataset, DatasetHeaderC, ExportVariant, Analysis
from util.codec import FileIn, FileOut, dataclassC, strC, intC
from util.codec_progress import CodecProgress, listCP, oneCP
from util.tree_model import Node

from dataclasses import dataclass

log = logging.getLogger(__name__)

@dataclass
class Subject:
    name : str

    # violation counts
    weak_stochastic_transitivity : int
    moderate_stochastic_transitivity : int
    strong_stochastic_transitivity : int

    regularity : int

SubjectC = dataclassC(Subject, strC, intC, intC, intC, intC)

class RootNode(util.tree_model.RootNode):
    def __init__(self, subjects: list[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects

    def create_child(self, row: int) -> 'SubjectNode':
        return SubjectNode(self, row, self.subjects[row])

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node : Node, row: int, subject: Subject) -> None:
        self.subject = subject

        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields = (
                subject.name,
                subject.weak_stochastic_transitivity,
                subject.moderate_stochastic_transitivity,
                subject.strong_stochastic_transitivity,
                subject.regularity,
            ),
        )

class StochasticConsistencyResult(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds: StochasticConsistencyResult) -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            # we assign model to prevent GC
            # even though we never read self.model
            self.model = util.tree_model.TreeModel(
                RootNode(ds.subjects),
                headers=(
                    'Subject',
                    'Weak Stochastic Transitivity',
                    'Moderate Stochastic Transitivity',
                    'Strong Stochastic Transitivity',
                    'Regularity',
                ),
            )
            self.twRows.setModel(self.model)

            hdr = self.twRows.header()

            hdr.setSectionsClickable(True)
            hdr.sectionClicked.connect(self.header_clicked)

            hdr.setSectionResizeMode(QHeaderView.ResizeToContents)
            hdr.setStretchLastSection(False)

        # not used ATM but leave this in; we'll probably add help later
        def header_clicked(self, idx : int) -> None:
            name = self.model.headers[idx].user_data
            if not name:
                return

            doc.open_in_browser(name)

    def __init__(self, name: str, alternatives: list[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: list[Subject] = []
        self.max_cycle_length : int = 0

    def get_analyses(self) -> Sequence[Analysis]:
        return []

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Summary',
                column_names=(
                    'subject',
                    'weak_stochastic_transitivity',
                    'moderate_stochastic_transitivity',
                    'strong_stochastic_transitivity',
                    'regularity',
                ),
                get_rows=self.export_summary,
                size=len(self.subjects),
            ),
        )

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    def export_summary(self) -> Iterator[Optional[
        tuple[str,int,int,int,int]
    ]]:
        for subject in self.subjects:
            yield (
                subject.name,
                subject.weak_stochastic_transitivity,
                subject.moderate_stochastic_transitivity,
                subject.strong_stochastic_transitivity,
                subject.regularity,
            )

            yield None

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress[StochasticConsistencyResult]:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : StochasticConsistencyResult) -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : StochasticConsistencyResult) -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)
            intC_encode(f, x.max_cycle_length)

        def decode(worker : Worker, f : FileIn) -> StochasticConsistencyResult:
            ds = StochasticConsistencyResult(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            ds.max_cycle_length = intC_decode(f)
            return ds

        return CodecProgress(get_size, encode, decode)
