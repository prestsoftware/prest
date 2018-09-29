import time
import random
import logging
import collections
from typing import List, FrozenSet, Set, NamedTuple

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QDialog, QMessageBox, QProgressDialog

import gui
import simulation
import uic.simulation
import dataset.experimental_data
from gui.progress import Worker, Cancelled

log = logging.getLogger(__name__)

class Options(NamedTuple):
    dataset_name : str
    alternatives : List[str]
    subject_count : int
    gen_menus : simulation.GenMenus
    gen_choices : simulation.GenChoices

class Simulation(QDialog, uic.simulation.Ui_Simulation, gui.ExceptionDialog):
    def __init__(self) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

        self.leAlternatives.textChanged.connect(self.catch_exc(self.update_alternatives))
        self.update_alternatives('')

        self.genMenus.cbDefault.currentIndexChanged.connect(
            self.catch_exc(self.default_changed)
        )
        self.default_changed(None)

    def default_changed(self, _index):
        self.genChoices.setDefault(
            self.genMenus.value().defaults,
            not self.genMenus.value().defaults,
        )

    def get_alternatives(self) -> List[str]:
        return [alt.strip() for alt in self.leAlternatives.text().split(',')]

    def update_alternatives(self, _text : str):
        alternatives = self.get_alternatives()
        self.genMenus.set_alt_count(len(alternatives))

    def value(self) -> Options:
        alternatives = self.get_alternatives()
        if len(alternatives) < 2:
            raise gui.ValidationError('Please specify at least two alternatives.')

        return Options(
            dataset_name=self.leName.text(),
            alternatives=alternatives,
            subject_count=self.sbSubjects.value(),
            gen_menus=self.genMenus.value(),
            gen_choices=self.genChoices.value(),
        )
