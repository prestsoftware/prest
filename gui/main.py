#!/usr/bin/env python3

# hide the console, first of all
if __name__ == '__main__':
    import sys
    sys.stdout.write('Prest is starting...\n')
    sys.stdout.flush()

import sys
import logging
import platform_specific

from PyQt5.QtWidgets import QApplication

import branding
import gui.main_window

logging.basicConfig(level=logging.DEBUG)
log = logging.getLogger(__name__)

main_win = None

def main(app):
    global main_win
    main_win = gui.main_window.MainWindow()
    main_win.show()
    exit_code = app.exec_()
    main_win.shutdown()
    sys.exit(exit_code)

if __name__ == '__main__':
    if platform_specific.is_windows():
        platform_specific.hide_console()
    app = QApplication(sys.argv)
    app.setApplicationName(branding.PREST_VERSION)
    app.setApplicationDisplayName(app.applicationName())
    main(app)
