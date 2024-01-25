import bz2
import logging
from typing import cast

import dataset
import dataset.budgetary
import dataset.experimental_data
import dataset.estimation_result
import dataset.deterministic_consistency_result
import dataset.stochastic_consistency_result
import dataset.experiment_stats

import branding
from gui.progress import Worker
from util.codec import FileIn, FileOut, strC, intC
from util.codec_progress import CodecProgress, listCP, enum_by_typenameCP

log = logging.getLogger(__name__)

PREST_SIGNATURE = b'Prest Workspace\0'
FILE_FORMAT_VERSION = 18

DatasetCP : CodecProgress = enum_by_typenameCP('Dataset', [
    (cls, cls.get_codec_progress())
    for cls in dataset.Dataset.__subclasses__()
])

class PersistenceError(Exception):
    pass

class Workspace:
    def __init__(self):
        self.datasets : list[dataset.Dataset] = []

    def save_to_file(self, worker : Worker, fname: str) -> None:
        with bz2.open(fname, 'wb') as f_raw:
            f = cast(FileOut, f_raw)  # assert we're doing output

            f.write(PREST_SIGNATURE)
            intC.encode(f, FILE_FORMAT_VERSION)  # version
            strC.encode(f, branding.VERSION)

            work_size = listCP(DatasetCP).get_size(self.datasets)
            intC.encode(f, work_size)

            worker.set_work_size(work_size)
            listCP(DatasetCP).encode(worker, f, self.datasets)

    def load_from_file(self, worker : Worker, fname: str) -> None:
        with bz2.open(fname, 'rb') as f_raw:
            f = cast(FileIn, f_raw)  # assert we're doing input

            sig = f.read(len(PREST_SIGNATURE))
            if sig != PREST_SIGNATURE:
                raise PersistenceError('not a Prest workspace file')

            version = intC.decode(f)
            if version >= 3:
                prest_version = strC.decode(f)
            else:
                prest_version = None  # too old

            if version != FILE_FORMAT_VERSION:
                message = 'incompatible PWF version: expected {0}, received {1}'.format(
                    FILE_FORMAT_VERSION,
                    version,
                )

                if prest_version:
                    message += ' (saved by {0})'.format(prest_version)

                raise PersistenceError(message)

            work_size = intC.decode(f)
            worker.set_work_size(work_size)
            datasets = listCP(DatasetCP).decode(worker, f)

        # assign to self only once everything's gone all right
        self.datasets = datasets
