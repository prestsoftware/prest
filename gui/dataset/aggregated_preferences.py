from __future__ import annotations

import logging
from gui.progress import Worker
from dataclasses import dataclass
from dataset import Dataset, Analysis, ExportVariant, DatasetHeaderC
from typing import Sequence, NewType
from util.codec import FileIn, FileOut, dataclassC, bytesC
from util.codec_progress import CodecProgress, oneCP

log = logging.getLogger(__name__)

InstanceRepr = NewType('InstanceRepr', bytes)
InstanceReprC = bytesC

@dataclass
class Response:
    instance : InstanceRepr

ResponseC = dataclassC(Response, InstanceReprC)

class AggregatedPreferences(Dataset):
    def __init__(
        self,
        name : str,
        alternatives : Sequence[str],
        response : Response,
    ) -> None:
        Dataset.__init__(self, name, alternatives)
        self.response = response

    def label_size(self) -> str:
        # not meaningful for this dataset
        return ''

    def get_analyses(self) -> Sequence[Analysis]:
        return ()

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return []

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress[AggregatedPreferences]:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        _get_size, response_encode, response_decode = oneCP(ResponseC).enc_dec()

        def get_size(_ : AggregatedPreferences) -> int:
            return 1

        def encode(worker : Worker, f : FileOut, x : AggregatedPreferences) -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            response_encode(worker, f, x.response)

        def decode(worker : Worker, f : FileIn) -> AggregatedPreferences:
            name, alternatives = DatasetHeaderC_decode(f)
            response = response_decode(worker, f)
            return AggregatedPreferences(name, alternatives, response)

        return CodecProgress(get_size, encode, decode)
