import logging
from typing import List, FrozenSet, Set, NamedTuple

from PyQt5.QtCore import Qt, QCoreApplication
from PyQt5.QtGui import QPixmap
from PyQt5.QtWidgets import QDialog, QMessageBox, QProgressDialog

import doc
import gui
import uic.about
import branding
import platform_specific

log = logging.getLogger(__name__)

class About(QDialog, uic.about.Ui_About, gui.ExceptionDialog):
    def __init__(self) -> None:
        QDialog.__init__(self)
        self.setupUi(self)

        self.lblPrest.setPixmap(
            QPixmap(
                platform_specific.get_embedded_file_path(
                    'images/prest-logo.png',      # deployment
                    'gui/images/prest-logo.png',  # development
                )
            )
        )
        self.lblVersion.setText(branding.VERSION)
        self.lblLicense.linkActivated.connect(self.catch_exc(self.open_license))

    def open_license(self, _url) -> None:
        doc.open_in_browser('copyright/index.html')
