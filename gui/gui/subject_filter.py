import time
import random
import logging
import collections
from dataclasses import dataclass
from typing import Any, Optional

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QWidget, QMessageBox, QProgressDialog

import gui
import uic.subject_filter
import dataset.experimental_data
import dataset.consistency_result
from gui.progress import Worker, Cancelled

import dataset
from core import Core

log = logging.getLogger(__name__)

@dataclass
class Options:
    run_consistency_analysis : bool
    condition_code : str

def accepts(options : Optional[Options], core : Core, subject_packed : dataset.PackedSubject) -> bool:
    if options is None:
        return True

    env : dict[str, Any] = {}

    if options.run_consistency_analysis:
        env['consistency'] = core.call(
            'consistency',
            dataset.PackedSubjectC,
            dataset.consistency_result.SubjectRawC,
            subject_packed
        )

    result : Any = eval(options.condition_code, env)
    assert isinstance(result, bool)  # SubjectFilter.value() checks this
    return result

class SubjectFilter(QWidget, uic.subject_filter.Ui_SubjectFilter):
    def __init__(self, parent : QWidget) -> None:
        QWidget.__init__(self, parent)
        self.setupUi(self)

    def value(self) -> Options:
        options = Options(
            run_consistency_analysis=self.cbConsistencyAnalysis.checkState() == Qt.Checked,
            condition_code=self.pteCondition.toPlainText(),
        )

        env = {}

        if options.run_consistency_analysis:
            env['consistency'] = dataset.consistency_result.SubjectRaw(
                name='subject',
                warp_pairs=0,
                warp_all=0,
                rows=[dataset.consistency_result.Row(
                    cycle_length=2,
                    garp=0,
                    sarp=0,
                    garp_binary_menus=0,
                    sarp_binary_menus=0,
                )],
            )

        try:
            result : Any = eval(options.condition_code, env)
        except Exception as e:
            raise gui.ValidationError(e)

        if not isinstance(result, bool):
            raise gui.ValidationError(f'This expression should compute a truth value but it computes {type(result)} instead.')

        return options
