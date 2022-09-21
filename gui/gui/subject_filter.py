import time
import random
import logging
import collections
from dataclasses import dataclass
from typing import List, FrozenSet, Set, NamedTuple

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtWidgets import QDialog, QMessageBox, QProgressDialog

import gui
import uic.subject_filter
import dataset.experimental_data
import dataset.consistency_result
from gui.progress import Worker, Cancelled

log = logging.getLogger(__name__)

@dataclass
class Options:
    run_consistency_analysis : bool
    condition_code : str

class SubjectFilter(uic.subject_filter.Ui_SubjectFilter, gui.ExceptionDialog):
    def __init__(self) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

    def value(self) -> Options:
        options = Options(
            run_consistency_analysis=self.cbConsistencyAnalysis.checkState() == Qt.Checked,
            condition_code=self.pteCondition.toPlainText(),
        )

        try:
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

            result : Any = eval(options.condition_code, env)

        except Exception as e:
            raise gui.ValidationError(e)

        if not isinstance(result, bool):
            raise gui.ValidationError(f'This expression should compute a truth value but it computes {type(result)} instead.')

        return options
