import logging
from typing import NamedTuple, List, Sequence, Iterator, Tuple, Optional, Set, FrozenSet, Union, cast
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView

import uic.view_dataset
import util.tree_model
from gui.progress import Worker
from dataset import Dataset, DatasetHeaderC, Analysis, ExportVariant
from util.codec import Codec, FileIn, FileOut, listC, strC, intC, \
    tupleC, namedtupleC, setC, frozensetC
from util.codec_progress import CodecProgress, listCP, oneCP

log = logging.getLogger(__name__)

class Row(NamedTuple):
    tuple_size : int
    garp_menu_tuples : Set[FrozenSet[FrozenSet[int]]]  # set of tuples of menus (= alt sets)

RowC = namedtupleC(Row, intC, setC(frozensetC(frozensetC(intC))))

class Subject(NamedTuple):
    name : str
    rows : List[Row]

SubjectC = namedtupleC(Subject, strC, listC(RowC))

class MenuRowNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, alternatives : List[str], xs : FrozenSet[FrozenSet[int]]) -> None:
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(
                '',
                '',
                '{' + ','.join(
                    '{' + ','.join(alternatives[i] for i in sorted(menu)) + '}'
                    for menu in sorted(xs)
                ) + '}',
            ),
        )

class RowNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, alternatives : List[str], r: Row) -> None:
        self.tuples = sorted(r.garp_menu_tuples)  # arbitrary but fixed order
        self.alternatives = alternatives

        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(
                '',
                r.tuple_size,
                len(self.tuples),
            ),
            child_count=len(self.tuples),
        )

    def create_child(self, row : int) -> MenuRowNode:
        return MenuRowNode(self, row, self.alternatives, self.tuples[row])

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, alternatives : List[str], subject: Subject) -> None:
        self.subject = subject
        self.alternatives = alternatives

        sum_menu = sum(len(r.garp_menu_tuples) for r in subject.rows)
        if subject.rows:
            len_min = min(r.tuple_size for r in subject.rows)
            len_max = max(r.tuple_size for r in subject.rows)
            len_range = f'{len_min} - {len_max}'
        else:
            # no intransitivities
            len_range = ''

        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(subject.name, len_range, sum_menu),
            child_count=len(subject.rows),
        )

    def create_child(self, row: int) -> RowNode:
        return RowNode(self, row, self.alternatives, self.subject.rows[row])

class RootNode(util.tree_model.RootNode):
    def __init__(self, alternatives : List[str], subjects : List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects
        self.alternatives = alternatives

    def create_child(self, row: int) -> SubjectNode:
        return SubjectNode(self, row, self.alternatives, self.subjects[row])

class TupleIntransMenus(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds : 'TupleIntransMenus') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.ds = ds
            self.model = util.tree_model.TreeModel(
                RootNode(ds.alternatives, ds.subjects),
                headers=(
                    'Subject',
                    'Tuple size',
                    'Menu tuples',
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
        return '%d subjects' % len(self.subjects)

    def get_analyses(self) -> Sequence[Analysis]:
        return ()

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Detailed',
                column_names=(
                    'subject',
                    'tuple_length',
                    'menus',
                ),
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        )

    def export_detailed(self) -> Iterator[Optional[Tuple[str,Optional[int],str]]]:
        for subject in self.subjects:
            # export a placeholder for subjects with no intransitivities
            if not subject.rows:
                yield (subject.name, None, '')
                yield None  # bump progress
                continue

            for row in subject.rows:
                for menu_tuple in sorted(row.garp_menu_tuples):
                    yield (subject.name, row.tuple_size,
                        ','.join(
                            '{' + ','.join(self.alternatives[i] for i in sorted(menu)) + '}'
                            for menu in sorted(menu_tuple)
                        ),
                    )

            yield None  # bump progress

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC))
        intC_encode, intC_decode = intC

        def get_size(x : 'TupleIntransMenus') -> int:
            return cast(int, subjects_size(x.subjects))

        def encode(worker : Worker, f : FileOut, x : 'TupleIntransMenus') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'TupleIntransMenus':
            ds = TupleIntransMenus(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
