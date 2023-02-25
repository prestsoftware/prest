from PyQt5.QtWidgets import QDialog

from enum import Enum
from uic.aggregate import Ui_Aggregate
from util.codec import pythonEnumC, strC

class Mode(Enum):
    Weighted = 'weighted'
    Iterated = 'iterated'

ModeC = pythonEnumC(Mode, strC)

class ConfigAggregated(QDialog, Ui_Aggregate):
    def __init__(self) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

    def value(self) -> Mode:
        if self.rbIterated.isChecked():
            return Mode.Iterated
        elif self.rbWeighted.isChecked():
            return Mode.Weighted
        else:
            raise Exception('no aggregation mode selected')
