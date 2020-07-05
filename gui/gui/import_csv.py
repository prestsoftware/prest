import os
import logging
from typing import Set, Sequence, List, Optional

from PyQt5.QtWidgets import QTableWidgetItem, QHeaderView, QDialog, QFileDialog, QMessageBox

import gui
import dataset
import dataset.experimental_data
import uic.import_csv
from dataset import SubjectC

log = logging.getLogger(__name__)

class ImportCsv(QDialog, uic.import_csv.Ui_ImportCsv, gui.ExceptionDialog):
    def __init__(self, main_win):
        QDialog.__init__(self)
        self.setupUi(self)

        self.main_win = main_win

        self.tblPreview.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeToContents)
        self.tblPreview.horizontalHeader().setStretchLastSection(False)

        self.rows: Optional[List[List[str]]] = None
        self.column_names: Optional[List[str]] = None

    def fill_rows(self, rows: List[List[str]]):
        # the UI will have warned the user otherwise so this must be true
        assert rows

        self.column_names = rows[0]
        self.rows = rows[1:]

        def fill_cols(cb, allow_none=False):
            assert self.column_names is not None
            cb.clear()
            assert self.column_names is not None
            for col in self.column_names:
                cb.addItem(col)
            if allow_none:
                cb.addItem('(none)')

        def preview(_arg=None):
            try:
                ds = self.make_dataset()
            except Exception as e:
                QMessageBox.warning(self, 'Bad CSV format', str(e))
                self.lwAlternatives.clear()
                self.tblPreview.setRowCount(0)
                return

            assert self.rows is not None

            self.lwAlternatives.clear()
            for alt in ds.alternatives:
                self.lwAlternatives.addItem(alt)

            # then grid preview
            MAX_ROWS = 256
            assert self.rows is not None
            nrows = min(len(self.rows), MAX_ROWS)
            self.tblPreview.setRowCount(nrows)
            #self.tblPreview.setColumnCount(len(self.column_names))
            #self.tblPreview.setHorizontalHeaderLabels(self.column_names)

            i = 0
            self.tblPreview.horizontalHeader().setSectionResizeMode(QHeaderView.Fixed)  # autoresize is **SLOW**
            for subj in map(SubjectC.decode_from_memory, ds.subjects):
                for cr in subj.choices:
                    self.tblPreview.setItem(i, 0, QTableWidgetItem(subj.name))
                    self.tblPreview.setItem(i, 1, QTableWidgetItem(subj.csv_set(cr.menu)))
                    self.tblPreview.setItem(i, 2, QTableWidgetItem(subj.csv_alt(cr.default)))
                    self.tblPreview.setItem(i, 3, QTableWidgetItem(subj.csv_set(cr.choice)))

                    i += 1
                    if i == MAX_ROWS:
                        break  # too many rows for preview

            self.tblPreview.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeToContents)

        fill_cols(self.cbSubject)
        fill_cols(self.cbMenu)
        fill_cols(self.cbDefault, allow_none=True)
        fill_cols(self.cbChoice)

        if len(self.column_names) < 3:
            raise Exception('the CSV file must contain at least 3 columns')
        elif len(self.column_names) == 3:
            indices = 0, 1, 3, 2
        else:
            indices = 0, 1, 2, 3

        self.cbSubject.setCurrentIndex(indices[0])
        self.cbMenu.setCurrentIndex(indices[1])
        self.cbDefault.setCurrentIndex(indices[2])
        self.cbChoice.setCurrentIndex(indices[3])
        
        self.cbSubject.currentIndexChanged.connect(self.catch_exc(preview))
        self.cbMenu.currentIndexChanged.connect(self.catch_exc(preview))
        self.cbDefault.currentIndexChanged.connect(self.catch_exc(preview))
        self.cbChoice.currentIndexChanged.connect(self.catch_exc(preview))

        preview()

    def make_dataset(self, name='CSV preview') -> dataset.experimental_data.ExperimentalData:
        assert self.column_names is not None
        assert self.rows is not None

        indices = (
            self.cbSubject.currentIndex(),
            self.cbMenu.currentIndex(),
            self.cbDefault.currentIndex() if self.cbDefault.currentIndex() < len(self.column_names) else None,
            self.cbChoice.currentIndex(),
        )
        
        ds = dataset.experimental_data.ExperimentalData.from_csv(
            name=name,
            rows=self.rows,
            indices=indices,
        )

        if '' in ds.alternatives:
            raise Exception('dataset contains an alternative with an empty name')

        return ds

    # override
    def accept(self) -> None:
        try:
            ds = self.make_dataset()
        except Exception as e:
            QMessageBox.warning(self, 'Bad CSV format', str(e))
        else:
            QDialog.accept(self)

    def run(self):
        fname : Optional[str] = None

        def work():
            assert fname is not None
            ds = self.make_dataset(name=os.path.basename(fname))
            self.main_win.add_dataset(ds)
            
        fname, _something = QFileDialog.getOpenFileName(self, "Import CSV", filter="CSV files (*.csv)")
        if not fname:
            return

        rows = dataset.load_raw_csv(fname)
        if not rows:
            QMessageBox.warning(
                self,
                "CSV import",
                "The input file seems to be empty",
            )
            return

        self.fill_rows(rows)
        self.accepted.connect(self.catch_exc(work))
        self.exec_()
