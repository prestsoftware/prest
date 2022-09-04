from typing import NamedTuple, List, Sequence, Iterator, Tuple, Optional, cast
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView

import uic.view_dataset
import util.tree_model
from gui.progress import Worker
from dataset import Dataset, DatasetHeaderC, Analysis, ExportVariant
from util.codec import Codec, FileIn, FileOut, listC, strC, intC, namedtupleC
from util.codec_progress import CodecProgress, listCP, oneCP

class Subject(NamedTuple):
    name : str
    observations : int
    active_choices : int
    active_choices_binary : int
    deferrals : int

SubjectC = namedtupleC(Subject, strC, intC, intC, intC, intC)

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, subject: Subject) -> None:
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=subject,
            child_count=0,
        )

class RootNode(util.tree_model.RootNode):
    def __init__(self, subjects : List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects

    def create_child(self, row: int) -> SubjectNode:
        return SubjectNode(self, row, self.subjects[row])

class ExperimentStats(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds : 'ExperimentStats') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.ds = ds
            self.model = util.tree_model.TreeModel(
                RootNode(ds.subjects),
                headers=('Subject', 'Observations', 'Active choices',
                    'Active choices in binary menus', 'Deferrals'),
            )
            self.twRows.setModel(self.model)

            self.twRows.header().setSectionResizeMode(QHeaderView.ResizeToContents)
            self.twRows.header().setStretchLastSection(False)

    def __init__(self, name : str, alternatives : Sequence[str] = ()) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects : List[Subject] = []

    def label_alts(self) -> str:
        return ''  # no alternatives in this dataset

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    def get_analyses(self) -> Sequence[Analysis]:
        return ()

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Detailed',
                column_names=(
                    'subject',
                    'observations',
                    'active_choices',
                    'active_choices_binary',
                    'deferrals',
                ),
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        )

    def export_detailed(self) -> Iterator[Optional[Tuple[str,int,int,int,int]]]:
        for subject in self.subjects:
            yield subject
            yield None  # bump progress

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : 'ExperimentStats') -> int:
            return cast(int, subjects_size(x.subjects))

        def encode(worker : Worker, f : FileOut, x : 'ExperimentStats') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'ExperimentStats':
            ds = ExperimentStats(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
