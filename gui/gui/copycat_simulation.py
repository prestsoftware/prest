import random
import logging
import collections
from typing import List, FrozenSet, Set, NamedTuple

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QDialog, QMessageBox, QProgressDialog

import gui
import simulation
import uic.copycat_simulation
from dataset import SubjectC
from gui.progress import Worker, Cancelled

log = logging.getLogger(__name__)

class Options(NamedTuple):
    name : str
    multiplicity : int
    gen_choices : 'simulation.GenChoices'

class CopycatSimulation(QDialog, uic.copycat_simulation.Ui_CopycatSimulation, gui.ExceptionDialog):
    def __init__(self, ds) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

        # this will be ExperimentalData but due to cyclic references,
        # we can't say that
        # ...and I want to keep this dialog in a separate file so whatever
        self.ds = ds

        self.update_counts(self.sbMultiplicity.value())
        self.sbMultiplicity.valueChanged.connect(self.catch_exc(self.update_counts))
        self.leName.setText(ds.name + ' (random choices)')

        class MyWorker(Worker):
            def work(self):
                has_defaults = False
                has_nondefaults = False

                self.set_work_size(len(ds.subjects))
                for i, subject in enumerate(map(SubjectC.decode_from_memory, ds.subjects)):
                    has_defaults |= any(cr.default is not None for cr in subject.choices)
                    has_nondefaults |= any(cr.default is None for cr in subject.choices)
                    self.set_progress(i)

                    if has_defaults and has_nondefaults:
                        break

                return has_defaults, has_nondefaults

        try:
            has_defaults, has_nondefaults = MyWorker().run_with_progress(
                None,  # parent widget
                'Checking presence of default alternatives...',
            )
            self.genChoices.setDefault(has_defaults, has_nondefaults)
        except Cancelled:
            self.genChoices.setDefault(True, True)
            log.debug('copycat simulation check cancelled')

    def update_counts(self, multiplicity : int) -> None:
        self.labSubjects.setText('{0} × {1} = {2}'.format(
            multiplicity,
            len(self.ds.subjects),
            multiplicity*len(self.ds.subjects),
        ))
        self.labObservations.setText('{0} × {1} = {2}'.format(
            multiplicity,
            self.ds.observ_count,
            multiplicity*self.ds.observ_count,
        ))

    def value(self) -> Options:
        return Options(
            name=self.leName.text(),
            multiplicity=self.sbMultiplicity.value(),
            gen_choices=self.genChoices.value(),
        )
