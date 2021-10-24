import json
import bz2
import logging
import sqlitebck
import sqlalchemy as sa
from dataclasses import dataclass
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
DB_SCHEMA_VERSION = 1

class PersistenceError(Exception):
    pass

@dataclass
class DatasetSummary:
    name : str
    type : str
    alternatives : List[str]
    size : str

class Workspace:
    def __init__(self):
        self.engine = sa.create_engine('sqlite://', future=True)  # in-memory DB
        with self.engine.connect() as db:
            dataset.metadata.create_all(db)

    def iter_dataset_summaries(self) -> Iterable[DatasetSummary]:
        with self.engine.connect() as db:
            dss = db.execute(
                sa.select([dataset.tbl_dataset])
                .order_by(dataset.tbl_dataset.c.id)
            ).all()

            for ds in dss:
                alternatives_db = db.execute(
                    sa.select([dataset.tbl_alternative.c.name])
                    .where(dataset.tbl_alternative.c.dataset_id == ds.id)
                    .order_by(dataset.tbl_alternative.c.id)
                )
                alternatives = [alt for alt, in alternatives_db]

                yield DatasetSummary(
                    name=ds.name,
                    alternatives=alternatives,
                    type=ds.type,
                    size='(TODO)',
                )

    def get_dataset_by_index(self, idx : int) -> dataset.Dataset:
        with self.engine.connect() as db:
            ds_db = db.execute(
                sa.select([dataset.tbl_dataset])
                .order_by(dataset.tbl_dataset.c.id)
                .offset(idx)
                .limit(1)
            ).one()

            alternatives_db = db.execute(
                sa.select([dataset.tbl_alternative.c.name])
                .where(dataset.tbl_alternative.c.dataset_id == ds_db.id)
                .order_by(dataset.tbl_alternative.c.id)
            )
            alternatives = [alt for alt, in alternatives_db]

            if ds_db.type == 'ExperimentalData':
                return dataset.experimental_data.ExperimentalData(
                    engine=self.engine,
                    name=ds_db.name,
                    alternatives=alternatives,
                    db_id=ds_db.id,
                )
            else:
                raise ValueError('unknown dataset type: ' + ds_db.type)

    def save_to_file(self, worker : Worker, fname : str) -> None:
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
