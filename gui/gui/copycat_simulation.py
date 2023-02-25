from __future__ import annotations

import random
import logging
import collections
from typing import Optional, TYPE_CHECKING
from dataclasses import dataclass

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QDialog, QMessageBox, QProgressDialog

import gui
import simulation
import gui.subject_filter
import uic.copycat_simulation
from dataset import SubjectC
from gui.progress import Worker, Cancelled

if TYPE_CHECKING:
    from dataset.experimental_data import ExperimentalData

log = logging.getLogger(__name__)

@dataclass
class Options:
    name : str
    multiplicity : int
    gen_choices : simulation.GenChoices
    preserve_deferrals : bool
    subject_filter : Optional[gui.subject_filter.Options]

class CopycatSimulation(uic.copycat_simulation.Ui_CopycatSimulation, gui.ExceptionDialog):
    def __init__(self, ds : ExperimentalData, experimental_features : bool) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

        self.ds = ds

        self.update_counts(self.sbMultiplicity.value())
        self.sbMultiplicity.valueChanged.connect(self.catch_exc(self.update_counts))
        self.leName.setText(ds.name + ' (random choices)')
        if not experimental_features:
            self.gbFilter.setVisible(False)

        class MyWorker(Worker):
            def work(self):
                has_defaults = False
                has_nondefaults = False
                has_deferrals = False

                self.set_work_size(len(ds.subjects))
                for i, subject in enumerate(map(SubjectC.decode_from_memory, ds.subjects)):
                    has_defaults |= any(cr.default is not None for cr in subject.choices)
                    has_nondefaults |= any(cr.default is None for cr in subject.choices)
                    has_deferrals |= any(not(cr.choice) for cr in subject.choices)
                    self.set_progress(i)

                    if has_defaults and has_nondefaults and has_deferrals:
                        break

                return has_defaults, has_nondefaults, has_deferrals

        try:
            has_defaults, has_nondefaults, has_deferrals = MyWorker().run_with_progress(
                None,  # parent widget
                'Analysing dataset...',
            )
            self.genChoices.setDefault(has_defaults, has_nondefaults)
        except Cancelled:
            self.genChoices.setDefault(True, True)
            log.debug('copycat simulation check cancelled')

        self.cbPreserveDeferrals.setChecked(False)
        self.cbPreserveDeferrals.setEnabled(has_deferrals)

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
            preserve_deferrals=self.cbPreserveDeferrals.isChecked(),
            subject_filter=
                self.subjectFilter.value()
                if self.gbFilter.isChecked()
                else None
        )
