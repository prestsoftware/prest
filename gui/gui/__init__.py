import re
import logging
import functools
from typing import Any, Callable

from PyQt5.QtWidgets import QMessageBox, QApplication, QDialog

import core

log = logging.getLogger(__name__)

class ValidationError(Exception):
    pass

class ExceptionDialog(QDialog):
    def catch_exc(self, f : Callable) -> Callable:
        @functools.wraps(f)
        def wrapper(*args : Any, **kwargs : Any) -> Any:
            try:
                return f(*args, **kwargs)

            except core.CoreDeath as e:
                # first guess -- the whole output
                error_message = str(e)

                # try to refine the message
                match = re.search(r"panicked at '(.*)', src/", error_message)
                if match:
                    error_message = match.group(1)

                QMessageBox.critical(
                    self,
                    "Internal error",
                    "Core computation failed:\n%s" % error_message
                )

            except core.MalformedResponse as e:
                QMessageBox.critical(
                    self,
                    "Internal error",
                    "Malformed response from core: %s" % e,
                )

            except ValidationError as e:
                log.info('validation error: %s', e)
                QMessageBox.warning(
                    self,
                    "Invalid data entered",
                    str(e),
                )

            except Exception as e:
                log.warning('exception in %s:', f)
                log.exception(str(e), exc_info=True)
                QMessageBox.warning(
                    self,
                    "Operation failed",
                    "Operation failed: %s" % e,
                )

        return wrapper

    def value(self) -> Any:
        return None

    def accept(self) -> None:
        try:
            _ = self.value()
        except ValidationError as e:
            QMessageBox.warning(self, "Invalid data entered", str(e))
        else:
            QDialog.accept(self)
