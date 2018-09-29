from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QWidget, QMessageBox, QProgressDialog

import simulation
from typing import Optional
from uic.gen_choices import Ui_GenChoices

class GenChoices(QWidget, Ui_GenChoices):
    def __init__(self, parent):
        QWidget.__init__(self, parent)
        self.setupUi(self)

    def setDefault(self, defaults : bool, nondefaults : bool) -> None:
        # to prevent showing both, which increases the size of the dialog
        self.gbDefault.setVisible(False)
        self.gbNoDefault.setVisible(False)

        self.gbDefault.setVisible(defaults)
        self.gbNoDefault.setVisible(nondefaults)

    def value(self) -> 'simulation.GenChoices':
        return simulation.Uniform(
            forced_choice=self.rbFC.isChecked() \
                or self.rbUnbiased.isChecked(),
            multiple_choice=self.cbMultipleChoice.isChecked(),
        )
