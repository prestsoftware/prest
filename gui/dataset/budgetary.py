import os
import dataset
import util.tree_model
import uic.view_dataset
import numpy as np

import dataset.budgetary_consistency
from core import Core
from dataset import Dataset, Analysis, ExportVariant, DatasetHeaderC
from dataset.budgetary_consistency import BudgetaryConsistency
from typing import Sequence, NamedTuple, List, Dict, Tuple, Iterator, Union, Optional, cast
from gui.progress import Worker, Cancelled
from util.codec import FileOut, FileIn, namedtupleC, strC, numpyC, listC
from util.codec_progress import CodecProgress, listCP, oneCP
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView

class Subject(NamedTuple):
    name    : str
    prices  : np.array
    amounts : np.array

SubjectC = namedtupleC(Subject, strC, numpyC(np.float32), numpyC(np.float32))

class RowNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, prices: np.array, amounts: np.array) -> None:
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=[''] \
                + ['%.2f' % p for p in prices] \
                + ['%g' % x for x in amounts] \
                + ['%.2f' % np.dot(prices, amounts)]
        )

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row: int, subject: Subject) -> None:
        self.subject = subject
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(subject.name,),
            child_count=subject.prices.shape[0],
        )

    def create_child(self, row: int) -> RowNode:
        return RowNode(self, row, self.subject.prices[row], self.subject.amounts[row])

class RootNode(util.tree_model.RootNode):
    def __init__(self, subjects : List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects

    def create_child(self, row: int) -> SubjectNode:
        return SubjectNode(self, row, self.subjects[row])

class Budgetary(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds : 'Budgetary') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.ds = ds
            self.model = util.tree_model.TreeModel(
                RootNode(ds.subjects),
                headers= \
                    ['Subject'] \
                    + [f'Price {a}' for a in ds.alternatives] \
                    + [f'Quantity {a}' for a in ds.alternatives] \
                    + ['Expenditure']
            )
            self.twRows.setModel(self.model)

            self.twRows.header().setSectionResizeMode(QHeaderView.ResizeToContents)
            self.twRows.header().setStretchLastSection(False)

    def __init__(self, name : str, alternatives : Sequence[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects : List[Subject] = []
        self.nr_observations = 0

    def label_size(self):
        return f'{len(self.subjects)} subjects, {self.nr_observations} observations'

    def analysis_consistency(self, worker : Worker, _config : None) -> BudgetaryConsistency:
        with Core() as core:
            worker.interrupt = lambda: core.shutdown()  # interrupt hook

            rows = []

            worker.set_work_size(len(self.subjects))
            for i, subject in enumerate(self.subjects):
                response = core.call(
                    'budgetary-consistency',
                    SubjectC,
                    dataset.budgetary_consistency.SubjectC,
                    subject
                )
                rows.append(response)

                worker.set_progress(i+1)

        ds = BudgetaryConsistency(
            self.name + ' (consistency)',
            self.alternatives,
            rows,
        )
        return ds

    def get_analyses(self) -> Sequence[Analysis]:
        return (
            Analysis(
                name='Consistency analysis',
                config=None,
                run=self.analysis_consistency,
            ),
        )

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC
        subjects_get_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC))

        def get_size(x : 'Budgetary') -> int:
            return cast(int, subjects_get_size(x.subjects))

        def encode(worker : Worker, f : FileOut, x : 'Budgetary') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'Budgetary':
            ds = Budgetary(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            ds.update_nr_observations()
            return ds

        return CodecProgress(get_size, encode, decode)

    def update_nr_observations(self) -> None:
        self.nr_observations = sum(len(s.prices) for s in self.subjects)


    def get_export_variants(self) -> Sequence[ExportVariant]:
        if not self.subjects:
            return []

        _n_observ, n_alts = self.subjects[0].prices.shape

        return [
            ExportVariant(
                name='Detailed',
                column_names=\
                    ['subject'] \
                    + [f'price{i+1}' for i in range(n_alts)] \
                    + [f'quantity{i+1}' for i in range(n_alts)],
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        ]

    def export_detailed(self) -> Iterator[Optional[Tuple[Union[str, float], ...]]]:
        for subject in self.subjects:
            # subject.prices and subject.amounts are matrices
            for prices, amounts in zip(subject.prices, subject.amounts):
                row : List[Union[str, float]] = [subject.name]
                row += list(prices)
                row += list(amounts)
                yield tuple(row)

            yield None  # bump progress

class BudgetaryError(Exception):
    pass

def load_from_csv(fname : str) -> Budgetary:
    lines = dataset.load_raw_csv(fname)
    if not lines:
        raise BudgetaryError("the CSV file is empty")

    header, *rows = lines

    if (len(header)-1) % 2 != 0:
        raise BudgetaryError("budgetary datasets should have an even number of numeric columns")

    n_alts = (len(header)-1) // 2
    alternatives = [f'{i+1}' for i in range(n_alts)]
    subjects : Dict[str,Tuple[List[np.array],List[np.array]]] = dict()  # ordered
    for line_no, row in enumerate(rows, start=2):
        if len(row) != len(header):
            raise BudgetaryError(f'{fname}, line {line_no}: incorrect number of columns')

        subj_name, *cols = row
        if subj_name not in subjects:
            subjects[subj_name] = ([], [])

        prices, amounts = subjects[subj_name]
        prices.append(np.array([float(x) for x in cols[:n_alts]], dtype=np.float32))
        amounts.append(np.array([float(x) for x in cols[n_alts:]], dtype=np.float32))

    ds = Budgetary(os.path.basename(fname), alternatives)
    ds.subjects = [
        Subject(name=n, prices=np.vstack(ps), amounts=np.vstack(ams))
        for (n,(ps,ams)) in subjects.items()
    ]
    ds.update_nr_observations()

    return ds
