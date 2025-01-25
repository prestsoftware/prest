import re
from typing import Sequence, List, Any, Optional, Callable, cast
from PyQt5.QtGui import QIcon
from PyQt5.QtCore import Qt, QModelIndex, QAbstractItemModel

REGEX_NUM = re.compile(r'^(?:[0-9]+|[0-9]*\.[0-9]+)$')

class Field:
    def __init__(self,
        text  : Optional[str] = None,
        icon  : Optional[QIcon] = None,
        user_data : Any = None,
        align : Optional[Qt.Alignment] = None,
    ) -> None:
        self.text = text
        self.icon = icon
        self.user_data = user_data

        if align:
            self.alignment = align
        else:
            if REGEX_NUM.match(text or ''):
                self.alignment = Qt.AlignRight | Qt.AlignVCenter
            else:
                self.alignment = Qt.AlignLeft | Qt.AlignVCenter

    def data(self, role : Qt.ItemDataRole) -> Any:
        if role == Qt.DisplayRole:
            return self.text
        elif role == Qt.DecorationRole:
            return self.icon
        elif role == Qt.UserRole:
            return self.user_data
        elif role == Qt.TextAlignmentRole:
            return self.alignment
        else:
            return None

def parse_fields(fields : Sequence[Any]) -> List[Field]:
    result : List[Field] = []

    for f in fields:
        if f is None:
            result.append(Field())
        elif isinstance(f, QIcon):
            result.append(Field(icon=f))
        elif isinstance(f, Field):
            result.append(f)
        else:
            result.append(Field(text=str(f)))

    return result

class Node:
    def __init__(self, parent_node: Optional['Node'], row: Optional[int], fields: Sequence[Any], child_count : int = 0) -> None:
        self.parent_node = parent_node
        self.row = row  # relative to parent
        self.child_count = child_count

        self.children: List[Optional[Node]] = [None] * self.child_count
        self.fields: Sequence[Field] = parse_fields(fields)

    def create_child(self, row: int) -> 'Node':
        raise NotImplementedError

    def field(self, column: int) -> Field:
        return self.fields[column]

    def child(self, row: int) -> 'Node':
        child = self.children[row]

        if child is None:
            child = self.create_child(row)
            self.children[row] = child

        return child

    def parent_idx(self, model : QAbstractItemModel) -> QModelIndex:
        if self.parent_node:
            assert self.row is not None
            return model.createIndex(self.row, 0, self.parent_node)
        else:
            return QModelIndex()

class RootNode(Node):
    def __init__(self, child_count: int) -> None:
        Node.__init__(
            self,
            parent_node=None,
            row=None,
            fields=[],
            child_count=child_count
        )

class TreeModel(QAbstractItemModel):
    def __init__(self, root: Node, headers: Sequence[Any]) -> None:
        QAbstractItemModel.__init__(self)
        self.root = root
        self.headers : List[Field] = parse_fields(headers)

    def headerData(self, nr: int, orientation: Qt.Orientation, role : int = Qt.DisplayRole) -> Optional[str]:  # type: ignore
        if orientation == Qt.Horizontal:
            try:
                return cast(
                    Optional[str],
                    self.headers[nr].data(cast(Qt.ItemDataRole, role)),
                )
            except IndexError:
                return None
        else:
            return None

    def get_node(self, idx: QModelIndex) -> Node:
        if idx.isValid():
            return cast(Node, idx.internalPointer())
        else:
            return self.root

    def index(self, row: int, col: int, idx: QModelIndex = QModelIndex()) -> QModelIndex:
        try:
            child = self.get_node(idx).child(row)
        except IndexError:
            return QModelIndex()

        return self.createIndex(row, col, child)

    def parent(self, idx: QModelIndex) -> Node:  # type: ignore
        return cast(Node, self.get_node(idx).parent_idx(self))

    def rowCount(self, idx: QModelIndex = QModelIndex()) -> int:
        return self.get_node(idx).child_count

    def columnCount(self, idx: QModelIndex = QModelIndex()) -> int:
        return len(self.headers)

    def data(self, idx: QModelIndex = QModelIndex(), role : int = Qt.DisplayRole) -> Optional[str]:  # type: ignore
        try:
            return cast(Optional[str], self.get_node(idx).field(idx.column()).data(cast(Qt.ItemDataRole, role)))
        except IndexError:
            return None

# -- utilities --

class _GroupNode(Node):
    parent_node : 'PackedRootNode'

    def __init__(self, parent_node : 'PackedRootNode', row: int) -> None:
        self.lo = row * parent_node.group_size  # zero-based, first item index
        hi = min((row + 1) * parent_node.group_size, len(parent_node.subjects))  # zero-based, first index AFTER
        self.parent_node : 'PackedRootNode' = parent_node

        Node.__init__(
            self, parent_node, row,
            fields=(
                '%s %d-%d' % (parent_node.subj_desc, self.lo+1, hi),  # one-based, inclusive (for humans)
                '', '', ''
            ),
            child_count = hi - self.lo,
        )

    def create_child(self, row : int) -> Any:
        return self.parent_node.subj_cls(
            self, row,
            self.parent_node.subj_decode(
                self.parent_node.subjects[self.lo + row]
            )
        )

class PackedRootNode(RootNode):
    def __init__(
        self,
        subj_cls : type,
        subj_decode : Callable[[bytes], Any],
        subj_desc : str,
        subjects: List,  # linear list of packed subjects!
    ) -> None:
        # distribute the subjects about evenly:
        # the number of groups should be roughly
        # group_factor times the number of subjects in a group
        #
        # group_factor exists since it's much cheaper to create groups
        # in the treeview than decode subjects within the group
        # so if you open a group, you shouldn't wait *too* long for decoding
        group_factor : int = 2
        group_size : int
        for size in (500, 1000, 5000):
            group_size = size
            if len(subjects) <= group_size*group_size*group_factor:
                break
        # let's be biased towards many groups rather than many subjects per group
        # if the above loop falls through, group_size will stay at the greatest value

        self.group_size = group_size
        self.subj_cls = subj_cls
        self.subj_decode = subj_decode
        self.subj_desc = subj_desc
        self.subjects = subjects

        group_count = (len(subjects) + group_size - 1) // group_size  # round up
        RootNode.__init__(self, group_count)

    def create_child(self, row: int) -> _GroupNode:
        return _GroupNode(self, row)
