from typing import Optional
from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QWidget, QMessageBox, QProgressDialog

import gui
import simulation
from uic.gen_menus import Ui_GenMenus

class GenMenus(QWidget, Ui_GenMenus):
    def __init__(self, parent : QWidget) -> None:
        QWidget.__init__(self, parent)
        self.setupUi(self)
        self.alt_count : Optional[int] = None

    def set_alt_count(self, alt_count : int) -> None:
        self.alt_count = alt_count

        self.lblExhaustiveCount.setText(str(2**alt_count - 1))
        self.lblBinaryCount.setText(str((alt_count * (alt_count - 1)) // 2))

    def value(self) -> 'simulation.GenMenus':
        assert self.alt_count is not None

        generator : simulation.MenuGenerator
        defaults  : bool

        if self.rbExhaustive.isChecked():
            generator = simulation.Exhaustive()

        elif self.rbSampleWithReplacement.isChecked():
            generator = simulation.SampleWithReplacement(
                menu_count=self.sbSampleWithReplacementCount.value(),
            )

        elif self.rbBinary.isChecked():
            generator = simulation.Binary()

        else:
            raise gui.ValidationError('please select a choice distribution')

        if self.cbDefault.currentIndex() == 0:
            defaults = False
        elif self.cbDefault.currentIndex() == 1:
            defaults = True
        else:
            raise gui.ValidationError('unrecognised choice of defaults')

        return simulation.GenMenus(
            generator=generator,
            defaults=defaults,
        )
