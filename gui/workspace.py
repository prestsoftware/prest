import json
import bz2
import logging
import sqlitebck
import sqlalchemy as sa
from typing import cast, List, Any, Iterable, BinaryIO

import dataset
import dataset.budgetary
import dataset.experimental_data
import dataset.estimation_result
import dataset.consistency_result
import dataset.experiment_stats

import branding
from gui.progress import Worker, Cancelled
from util.codec import Codec, FileIn, FileOut, strC, intC, listC
from util.codec_progress import CodecProgress, listCP, oneCP, enum_by_typenameCP

log = logging.getLogger(__name__)

PREST_SIGNATURE = 'Prest Workspace'
DB_FORMAT_VERSION = 1

class PersistenceError(Exception):
    pass

class Workspace:
    def __init__(self):
        self.engine = sa.create_engine('sqlite://', future=True)  # in-memory DB
        with self.engine.connect() as db:
            dataset.metadata.create_all(db)

    def get_datasets(self) -> List[dataset.Dataset]:
        return []  # todo

    def save_to_file(self, worker : Worker, fname: str) -> None:
        # TODO: signatures, versioning
        new_engine = sa.create_engine(f'sqlite:///{fname}', future=True)
        with self.engine.connect() as db:
            with new_engine.connect() as db2:
                db.connection.backup(
                    db2.connection.connection
                )
        self.engine = new_engine

    def load_from_file(self, worker : Worker, fname: str) -> None:
        self.engine = sa.create_engine(f'sqlite:///{fname}')
        # TODO: signatures, versioning
