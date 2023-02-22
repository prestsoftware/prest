# Mostly taken from:
# https://github.com/pyinstaller/pyinstaller/issues/1339#issuecomment-122909830

import os
import sys
import ctypes
import logging
from typing import Optional, cast, Callable

log = logging.getLogger(__name__)

class FileNotFound(Exception):
    def __init__(self, *fnames : str) -> None:
        if len(fnames) > 1:
            Exception.__init__(self, 'could not find embedded file: %s' % ', '.join(fnames))
        else:
            Exception.__init__(self, 'could not find embedded file: %s' % ', '.join(fnames))

def is_frozen() -> bool:
    return cast(bool, getattr(sys, 'frozen', False))

def is_windows() -> bool:
    return sys.platform.lower().startswith('win')

def hide_console() -> None:
    log.debug('hiding console...')
    if sys.platform == 'win32':
        whnd = ctypes.windll.kernel32.GetConsoleWindow()
        if whnd != 0:
            ctypes.windll.user32.ShowWindow(whnd, 0)
    else:
        raise Exception('console hiding unsupported on %s' % sys.platform)

def show_console() -> None:
    log.debug('showing console...')
    if sys.platform == 'win32':
        whnd = ctypes.windll.kernel32.GetConsoleWindow()
        if whnd != 0:
            ctypes.windll.user32.ShowWindow(whnd, 1)
    else:
        raise Exception('console showing unsupported on %s' % sys.platform)

def get_frozen_dir() -> str:
    result : Optional[str] = cast(
        Optional[str],
        getattr(sys, '_MEIPASS', None),
    )
    assert result is not None
    return result

# takes list of possible names
def get_embedded_file_path(*fnames : str, criterion : Callable[[str], bool] = os.path.isfile) -> str:
    for fname in fnames:
        path = os.path.join(os.getcwd(), fname)
        if criterion(path):
            return path  # in current directory

        if is_frozen():
            path = os.path.join(get_frozen_dir(), fname)
            if criterion(path):
                return path

    raise FileNotFound(*fnames)
