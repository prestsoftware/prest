from PyQt5.QtWidgets import QDialog

from enum import Enum
from uic.aggregate import Ui_Aggregate
from util.codec import pyEnumC, strC

class Mode(Enum):
    Weighted = 'weighted'
    Iterated = 'iterated'

ModeC = pyEnumC(Mode, strC)

class ConfigAggregated(QDialog, Ui_Aggregate):
    def __init__(self, chain_count : int, chain_length : int) -> None:
        QDialog.__init__(self)
        self.setupUi(self)
        self.lCount.setText(str(chain_count))
        self.lLength.setText(str(chain_length))

    def value(self) -> Mode:
        if self.rbIterated.isChecked():
            return Mode.Iterated
        elif self.rbWeighted.isChecked():
            return Mode.Weighted
        else:
            raise Exception('no aggregation mode selected')
