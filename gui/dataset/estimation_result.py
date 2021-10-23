import json
import collections
import hashlib
import base64
from typing import NamedTuple, Sequence, List, Iterator, Tuple, Dict, \
    Optional, Any, Union, NewType, cast, Callable

from PyQt5.QtGui import QIcon, QCursor
from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView, QToolTip

import gui
import model
import dataset
import platform_specific
from gui.progress import Worker
from model import get_name as model_get_name
from model import Model as ModelRepr
from model import ModelC
from model import get_ordering_key as model_get_ordering_key
from dataset import Dataset, DatasetHeaderC, ExportVariant, Analysis
from util.tree_model import Node, TreeModel, Field, PackedRootNode
from util.codec import Codec, FileIn, FileOut, namedtupleC, strC, intC, \
    frozensetC, listC, bytesC, tupleC, boolC
from util.codec_progress import CodecProgress, listCP, oneCP
import uic.view_estimated

class Penalty(NamedTuple):
    # both bounds are inclusive
    lower_bound: int
    upper_bound: int

    def __str__(self):
        return str(self.to_csv())

    def to_csv(self) -> Union[int, str]:
        if self.lower_bound != self.upper_bound:
            return f'{self.lower_bound-1} < N ≤ {self.upper_bound}'
        else:
            return self.upper_bound

PenaltyC = namedtupleC(Penalty, intC, intC)

class Request(NamedTuple):
    subjects : List[dataset.PackedSubject]
    models : Sequence[model.Model]
    disable_parallelism : bool
    disregard_deferrals : bool

RequestC = namedtupleC(Request, listC(dataset.PackedSubjectC), listC(ModelC), boolC, boolC)

InstanceRepr = NewType('InstanceRepr', bytes)
InstanceReprC = bytesC

class InstanceInfo(NamedTuple):
    model : ModelRepr
    penalty : Penalty
    instance : InstanceRepr

InstanceInfoC = namedtupleC(InstanceInfo, ModelC, PenaltyC, InstanceReprC)

class Response(NamedTuple):
    subject_name : str
    penalty : Penalty
    best_instances : List[InstanceInfo]

ResponseC = namedtupleC(Response, strC, PenaltyC, listC(InstanceInfoC))
ResponsesC = listC(ResponseC)

PackedResponse = NewType('PackedResponse', bytes)
PackedResponseC = bytesC
PackedResponsesC = listC(PackedResponseC)

class Instance(NamedTuple):
    model: str
    data: InstanceRepr

InstanceC = namedtupleC(Instance, strC, InstanceReprC)

class Subject(NamedTuple):
    name: str
    penalty: Penalty
    best_models: List[Tuple[model.Model, Penalty, List[InstanceRepr]]]

SubjectC = namedtupleC(Subject, strC, PenaltyC, listC(tupleC(ModelC, PenaltyC, listC(InstanceReprC))))

PackedSubject = NewType('PackedSubject', bytes)
PackedSubjectC = bytesC

def subject_from_response_bytes(response_bytes : PackedResponse) -> Subject:
    # returns something orderable
    def model_sort_criterion(chunk : Tuple[ModelRepr, Tuple[Penalty, List[InstanceRepr]]]) -> Any:
        model, (penalty, instances) = chunk
        return (
            model_get_ordering_key(model),
            len(instances),
            len(model_get_name(model)),
        )

    subject_name, subject_penalty, best_instances = ResponseC.decode_from_memory(response_bytes)

    by_model: Dict[model.Model, Tuple[Penalty, List[InstanceRepr]]] = {}
    for model, inst_penalty, instance in best_instances:
        penalty_instances_so_far = by_model.get(model)
        if penalty_instances_so_far:
            penalty_so_far, instances_so_far = penalty_instances_so_far
            assert penalty_so_far == inst_penalty, f'assertion failed: {inst_penalty} ≠ {penalty_so_far}'
            instances_so_far.append(instance)
        else:
            by_model[model] = (inst_penalty, [instance])

    return Subject(
        name=subject_name,
        penalty=subject_penalty,
        best_models=[
            (model, penalty, instances)
            for model, (penalty, instances)
            in sorted(by_model.items(), key=model_sort_criterion)
        ],
    )

class EstimationResult(Dataset):
    class Subject(Node):
        def __init__(self, parent_node, row: int, subject: Subject) -> None:
            Node.__init__(
                self, parent_node, row,
                fields=(subject.name, str(subject.penalty), '%d models' % len(subject.best_models)),
                child_count=len(subject.best_models),
            )
            self.subject = subject

        def create_child(self, row: int) -> 'EstimationResult.Model':
            model, penalty, instances = self.subject.best_models[row]
            return EstimationResult.Model(self, row, model, penalty, instances)

    class Model(Node):
        def __init__(self, parent_node: 'EstimationResult.Subject', row: int,
            model: model.Model, penalty: Penalty, instances: List[InstanceRepr]
        ) -> None:
            subject = parent_node.subject
            Node.__init__(
                self, parent_node, row,
                fields=(model_get_name(model), penalty, '%d instances' % len(instances)),
                child_count=len(instances),
            )
            self.instances = instances
            self.subject = subject

        def create_child(self, row: int) -> 'EstimationResult.Instance':
            return EstimationResult.Instance(self, row, self.instances[row])

    class Instance(Node):
        def __init__(self, parent_node: 'EstimationResult.Model', row: int, instance: InstanceRepr) -> None:
            code = base64.b64encode(instance).decode('ascii')
            subject = parent_node.subject
            #help_icon = QIcon(platform_specific.get_embedded_file_path('images/qm-16.png'))
            Node.__init__(
                self, parent_node, row,
                #fields=(code, Field(icon=help_icon, user_data=code), ''),
                fields=(code, '', ''),
            )

    class ViewDialog(uic.view_estimated.Ui_ViewEstimated, gui.ExceptionDialog):
        def __init__(self, ds: 'EstimationResult') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.model = TreeModel(
                PackedRootNode(
                    EstimationResult.Subject,
                    cast(Callable[[bytes], Any], subject_from_response_bytes),
                    'Subject',
                    ds.subjects
                ),
                headers=('Name', 'Distance score', 'Size'),
            )
            self.twSubjects.setModel(self.model)
            self.twSubjects.header().setSectionResizeMode(QHeaderView.ResizeToContents)
            self.twSubjects.header().setStretchLastSection(False)

            self.twSubjects.clicked.connect(self.catch_exc(self.dlg_item_clicked))

        def dlg_item_clicked(self, idx):
            code = self.model.data(idx, Qt.UserRole)
            if code:
                QToolTip.showText(QCursor.pos(), "This popup will show a visualisation of instance %s once it's implemented." % code)

    def __init__(self, name: str, alternatives: Sequence[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: List[PackedResponse] = []

    def get_analyses(self) -> Sequence[Analysis]:
        return []

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return (
            ExportVariant(
                name='Compact (human-friendly)',
                column_names=('subject', 'dist_score', 'model', 'instances'),
                get_rows=self.export_compact,
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Detailed (machine-friendly)',
                column_names=('subject', 'dist_score', 'dist_score_upper_bound', 'model', 'instance'),
                get_rows=self.export_detailed,
                size=len(self.subjects),
            ),
        )

    def export_detailed(self) -> Iterator[Optional[Tuple[str,Optional[int],int,str,str]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            for model, penalty, instances in subject.best_models:
                for instance in sorted(instances):
                    yield (
                        subject.name,
                        penalty.lower_bound if penalty.lower_bound == penalty.upper_bound else None,
                        penalty.upper_bound,
                        model_get_name(model),
                        base64.b64encode(instance).decode('ascii')
                    )

            yield None  # bump progress

    def export_compact(self) -> Iterator[Optional[Tuple[Optional[str],Union[int,str],str,int]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            subject_name: Optional[str] = subject.name
            for model, model_penalty, instances in subject.best_models:
                yield (subject_name, model_penalty.to_csv(), model_get_name(model), len(instances))
                subject_name = None  # don't repeat these

            yield None  # bump progress

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        subjects_encode : Callable[[Worker, FileOut, List[PackedResponse]], None]
        subjects_decode : Callable[[Worker, FileIn], List[PackedResponse]]

        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(PackedResponseC))
        intC_encode, intC_decode = intC

        def get_size(x : 'EstimationResult') -> int:
            return cast(int, subjects_size(x.subjects))

        def encode(worker : Worker, f : FileOut, x : 'EstimationResult') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'EstimationResult':
            ds = EstimationResult(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
