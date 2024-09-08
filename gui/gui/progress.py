# mypy can't see that exceptions may be set from other threads
# mypy: no-warn-unreachable

import logging
import collections
from typing import Any, List, Callable, Optional

from PyQt5.QtGui import QIcon, QCursor
from PyQt5.QtCore import Qt, QObject, QCoreApplication, QSize, QThread, pyqtSignal, pyqtSlot
from PyQt5.QtWidgets import QHeaderView, QDialog, QMessageBox, QTreeWidgetItem, QProgressDialog, QToolTip

import gui
import model
import uic.progress
import platform_specific
from core import Core

log = logging.getLogger(__name__)

ZOMBIES : List[QThread] = []

class ProgressDialog(uic.progress.Ui_Progress, gui.ExceptionDialog):
    def __init__(self, main_win, label_text: str) -> None:
        QDialog.__init__(self, main_win)
        self.setupUi(self)

        self.label.setText(label_text)
        self.progressBar.setMaximum(0)
        self.progressBar.setValue(0)

    @pyqtSlot(int)
    def set_maximum(self, value: int) -> None:
        self.progressBar.setMaximum(value)

    @pyqtSlot(int)
    def set_position(self, value: int) -> None:
        self.progressBar.setValue(value)

class WorkerError(Exception):
    pass

class Cancelled(Exception):
    pass

class Worker(QThread):
    work_size = pyqtSignal(int)
    progress = pyqtSignal(int)

    def __init__(self, *args) -> None:
        QObject.__init__(self)
        self.result : Optional[Any] = None
        self.exception : Optional[Exception] = None
        self.args = args
        self.position = 0

        # a lambda that can be called from the main thread
        # to interrupt the thread (e.g. close a socket or so)
        #
        # it's not a method because the closure may have to be
        # dynamically created (e.g. core connection)
        self.interrupt : Optional[Callable[[], None]] = None

    # called from work thread
    def set_work_size(self, value: int) -> None:
        self.work_size.emit(value)

    # called from work thread
    def set_progress(self, value: int) -> None:
        if self.isInterruptionRequested():
            log.debug('worker: interruption requested, cancelling...')
            raise Cancelled

        self.progress.emit(value)
        self.position = value

    # called from work thread
    def step(self) -> None:
        self.set_progress(self.position+1)

    # called from work thread
    def work(self, *args : Any) -> Any:
        # this is the method that should be overriden
        raise NotImplementedError()

    # overridden from QThread
    # called from work thread
    def run(self):
        try:
            self.result = self.work(*self.args)
        except Exception as e:
            self.exception = e

    # called from main application thread
    def run_with_progress(self, main_win, label_text: str) -> Any:
        progress: ProgressDialog = ProgressDialog(main_win, label_text)

        self.work_size.connect(progress.catch_exc(progress.set_maximum))
        self.progress.connect(progress.catch_exc(progress.set_position))
        progress.rejected.connect(progress.catch_exc(self.requestInterruption))
        self.finished.connect(progress.catch_exc(progress.accept))

        log.debug('launching worker...')

        self.start()
        progress.exec()

        log.debug('event loop of progress bar terminated, waiting for worker')

        interrupted = False
        success = self.wait(2000)  # msec
        if success:
            log.debug('worker joined all right')
        else:
            log.warning("thread won't terminate, invoking job interruption...")
            if self.interrupt is not None:
                self.interrupt()
                interrupted = True

                log.debug("worker interrupted, waiting a bit more")
                success = self.wait(2000)
            else:
                log.debug("the worker does not implement interrupts")
                success = False

            if success:
                log.debug("worker eventually joined")
            else:
                log.error("worker won't join, terminating the whole thread forcibly")
                self.terminate()
                success = self.wait(2000)  # give it 2 more seconds and pray
                if not success:
                    log.error("terminate did not work, just pray now, I guess")

                # if the thread is not done yet at this point,
                # the program will probably crash when the QThread goes out of scope
                # and is garbage collected
                # (Qt doc explicitly disallows destruction while the thread is running)
                #
                # we could probably prevent this by pushing the reference to the QThread
                # into a global list of zombie threads to prevent their GC,
                # if it turns out to be a problem

        if self.exception is None:
            return self.result
        elif isinstance(self.exception, Cancelled):
            raise Cancelled()
        else:
            if interrupted:
                # this exception was likely caused by killing the core
                # via worker.interrupt(), after the user requested cancel
                # let's ignore it
                raise Cancelled()
            else:
                raise self.exception

class MockWorker(Worker):
    def __init__(self):
        pass

    def set_work_size(self, _size : int) -> None:
        pass

    def set_progress(self, _value : int) -> None:
        pass
