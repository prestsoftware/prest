import logging

from PyQt5.QtGui import QIcon
from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import QHeaderView, QDialog, QMessageBox, QTreeWidgetItem, \
    QPushButton, QLabel, QCheckBox, QHBoxLayout, QWidget

import doc
import gui
import model
import uic.estimation
import platform_specific
from util.codec import pyEnumC, strC
from model import Model
from dataclasses import dataclass
from enum import Enum

log = logging.getLogger(__name__)

class DistanceScore(Enum):
    HOUTMAN_MAKS = 'houtman-maks'
    JACCARD = 'jaccard'

distanceScoreC = pyEnumC(DistanceScore, strC)

@dataclass
class Options:
    models : list[Model]
    disable_parallelism : bool
    disregard_deferrals : bool
    distance_score : DistanceScore

class Estimation(uic.estimation.Ui_Estimation, gui.ExceptionDialog):
    def __init__(self):
        QDialog.__init__(self)
        self.setupUi(self)

        self.checkboxes : list[tuple[QCheckBox, Model]] = []

        self.fill_table()
        self.twModels.expandAll()

        header = self.twModels.header()
        assert header
        header.setStretchLastSection(False)
        header.setSectionResizeMode(QHeaderView.ResizeToContents)

    def fill_table(self):
        self.checkboxes = []
        help_icon = QIcon(platform_specific.get_embedded_file_path(
            'images/qm-16.png',      # deploy
            'gui/images/qm-16.png',  # devel
        ))
        self.twModels.clear()

        def add_item(parent, item):
            if isinstance(item, model.Category):
                twi = QTreeWidgetItem(parent, [item.name, '', ''])
                twi.setFirstColumnSpanned(True)
                for child in item.children:
                    add_item(twi, child)

            elif isinstance(item, model.ModelGroup):
                twi = QTreeWidgetItem(parent)
                twi.setDisabled(not item.variants)

                cell = QWidget()
                layout = QHBoxLayout(cell)
                layout.setContentsMargins(4,2,4,2)  # l,t,r,b

                lblName = QLabel(item.name)
                layout.addWidget(lblName, alignment=Qt.AlignmentFlag.AlignVCenter)

                self.twModels.setItemWidget(twi, 1, cell)

                if item.help_url:
                    btn = QPushButton(help_icon, '')
                    btn.setFlat(True)

                    # create a closure for `html`
                    def connect(url):
                        btn.clicked.connect(self.catch_exc(
                            lambda _checked: doc.open_in_browser(url)
                        ))
                    connect(item.help_url)

                    self.twModels.setItemWidget(twi, 0, btn)

                for i, name_model in enumerate(item.variants, 2):
                    if name_model is None:
                        continue

                    # the identifier "model" would clash with the module import
                    name_html, model_def = name_model

                    cb = QCheckBox()
                    label = QLabel(name_html)
                    label.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter)

                    cell = QWidget()
                    layout = QHBoxLayout(cell)
                    layout.setSpacing(4)
                    layout.setContentsMargins(4,0,4,0)  # l,t,r,b
                    layout.addWidget(cb, stretch=0, alignment=Qt.AlignVCenter)
                    layout.addWidget(label, stretch=1, alignment=Qt.AlignVCenter)
                    # all extra space (stretch) goes to the label because stretch ratio is 1:0

                    self.twModels.setItemWidget(twi, i, cell)
                    self.checkboxes.append((cb, model_def))

            else:
                raise Exception('unknown item: %s' % item)

        root = self.twModels.invisibleRootItem()
        for item in model.MODELS:
            add_item(root, item)

    def value(self) -> Options:
        return Options(
            models=[model for cb, model in self.checkboxes if cb.isChecked()],
            disable_parallelism=self.cbDisableParallelism.isChecked(),
            disregard_deferrals=self.cbDisregardDeferrals.isChecked(),
            distance_score=[
                DistanceScore.HOUTMAN_MAKS,
                DistanceScore.JACCARD,
            ][self.cbDistanceScore.currentIndex()],
        )

    # override from QDialog
    def accept(self) -> None:
        models = self.value().models
        if not models:
            QMessageBox.warning(
                self,
                'Please select models',
                'Please select at least one model for estimation',
            )

        elif models == [model.SequentiallyRationalizableChoice()]:
            QMessageBox.warning(
                self,
                'Please select additional models',
                'Sequentially Rationalizable Choice is an experimental model '
                'and cannot be selected alone. Please select at least one another model.'
            )

        else:
            QDialog.accept(self)
