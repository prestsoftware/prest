import logging
import util.tree_model
import uic.view_dataset
from gui.progress import Worker
from dataset import Dataset, Analysis, ExportVariant, DatasetHeaderC
from util.codec import namedtupleC, setC, intC, enumC, strC
from typing import NamedTuple, Set, Union, List, Sequence
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView
from util.codec import Codec, FileIn, FileOut, listC, strC, intC, \
    tupleC, namedtupleC, setC, frozensetC
from util.codec_progress import CodecProgress, listCP, oneCP

log = logging.getLogger(__name__)

class RepeatedMenu(NamedTuple):
    menu : Set[int]
    tag : int = 0

class ChoiceNotInMenu(NamedTuple):
    menu : Set[int]
    choice : int
    tag : int = 1

Issue = Union[
    RepeatedMenu,
    ChoiceNotInMenu,
]

IssueC = enumC('Issue', {
    RepeatedMenu: (setC(intC),),
    ChoiceNotInMenu: (setC(intC), intC),
})

class Subject(NamedTuple):
    name : str
    issues : List[Issue]

SubjectC = namedtupleC(Subject, strC, listC(IssueC))

class IssueNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, alternatives : List[str], issue : Issue) -> None:
        if isinstance(issue, RepeatedMenu):
            fields = (
                'repeated menu',
                ','.join(alternatives[i] for i in sorted(issue.menu)),
                '',
            )
        elif isinstance(issue, ChoiceNotInMenu):
            fields = (
                'choice not in menu',
                ','.join(alternatives[i] for i in sorted(issue.menu)),
                alternatives[issue.choice],
            )
        else:
            raise Exception('bad instance: %r' % issue)

        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=fields,
        )

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, alternatives : List[str], subject: Subject) -> None:
        self.subject = subject
        self.alternatives = alternatives

        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(subject.name, '', ''),
            child_count=len(subject.issues),
        )

    def create_child(self, row: int) -> IssueNode:
        return IssueNode(self, row, self.alternatives, self.subject.issues[row])

class RootNode(util.tree_model.RootNode):
    def __init__(self, alternatives : List[str], subjects : List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects
        self.alternatives = alternatives

    def create_child(self, row: int) -> SubjectNode:
        return SubjectNode(self, row, self.alternatives, self.subjects[row])

class IntegrityCheck(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds : 'IntegrityCheck') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.ds = ds
            self.model = util.tree_model.TreeModel(
                RootNode(ds.alternatives, ds.subjects),
                headers=(
                    'Subject',
                    'Menu',
                    'Choice',
                ),
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
        return '%d issues' % sum(len(subj.issues) for subj in self.subjects)

    def get_analyses(self) -> Sequence[Analysis]:
        return ()

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return []

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC))
        intC_encode, intC_decode = intC

        def get_size(x : 'IntegrityCheck') -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : 'IntegrityCheck') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'IntegrityCheck':
            ds = IntegrityCheck(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
