import base64
from fractions import Fraction
from typing import NamedTuple, Sequence, Iterator, \
    Optional, Any, NewType, cast, Callable

from PyQt5.QtGui import QIcon
from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import QDialog, QHeaderView, \
    QMessageBox, QFileDialog

import gui
import model
import dataset
import subprocess
import platform_specific
from dataclasses import dataclass
from core import Core
from gui.progress import Worker
from gui.estimation import DistanceScore, distanceScoreC
from model import get_name as model_get_name
from model import Model as ModelRepr
from model import ModelC
from model import get_ordering_key as model_get_ordering_key
from dataset import Dataset, DatasetHeaderC, ExportVariant, Analysis
from util.tree_model import Node, TreeModel, Field, PackedRootNode
from util.codec import Codec, FileIn, FileOut, namedtupleC, strC, intC, \
    frozensetC, listC, bytesC, tupleC, boolC, fractionC
from util.codec_progress import CodecProgress, listCP, oneCP
import uic.view_estimated

def from_fraction(x : Fraction) -> int | float:
    if x.denominator == 1:
        return x.numerator
    else:
        return round(float(x), 3)  # sounds about appropriate

class Penalty(NamedTuple):
    # both bounds are inclusive
    lower_bound: Fraction
    upper_bound: Fraction

    def __str__(self):
        return str(self.to_csv())

    def to_csv(self) -> str | int | float:
        if self.lower_bound != self.upper_bound:
            return f'{self.lower_bound-1} < N ≤ {self.upper_bound}'
        else:
            return from_fraction(self.upper_bound)

PenaltyC = namedtupleC(Penalty, fractionC, fractionC)

class Request(NamedTuple):
    subjects : list[dataset.PackedSubject]
    models : Sequence[model.Model]
    disable_parallelism : bool
    disregard_deferrals : bool
    distance_score : DistanceScore

RequestC = namedtupleC(Request, listC(dataset.PackedSubjectC), listC(ModelC), boolC, boolC, distanceScoreC)

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
    best_instances : list[InstanceInfo]

ResponseC = namedtupleC(Response, strC, PenaltyC, listC(InstanceInfoC))
ResponsesC = listC(ResponseC)

PackedResponse = NewType('PackedResponse', bytes)
PackedResponseC = cast(Codec[PackedResponse], bytesC)
PackedResponsesC = listC(PackedResponseC)

class InstVizRequest(NamedTuple):
    instance_code : str

InstVizRequestC = namedtupleC(InstVizRequest, strC)

class GraphRepr(NamedTuple):
    vertices : list[frozenset[int]]
    edges : list[tuple[frozenset[int], frozenset[int]]]

GraphReprC = namedtupleC(GraphRepr, listC(frozensetC(intC)), listC(tupleC(frozensetC(intC), frozensetC(intC))))

class InstVizResponse(NamedTuple):
    graphs : list[GraphRepr]
    extra_info : list[tuple[str, str]]

InstVizResponseC = namedtupleC(InstVizResponse, listC(GraphReprC), listC(tupleC(strC, strC)))

class Instance(NamedTuple):
    model: str
    data: InstanceRepr

InstanceC = namedtupleC(Instance, strC, InstanceReprC)

class Subject(NamedTuple):
    name: str
    penalty: Penalty
    best_models: list[tuple[model.Model, Penalty, list[InstanceRepr]]]

SubjectC = namedtupleC(Subject, strC, PenaltyC, listC(tupleC(ModelC, PenaltyC, listC(InstanceReprC))))

PackedSubject = NewType('PackedSubject', bytes)
PackedSubjectC = cast(Codec[PackedSubject], bytesC)

def subject_from_response_bytes(response_bytes : PackedResponse) -> Subject:
    # returns something orderable
    def model_sort_criterion(chunk : tuple[ModelRepr, tuple[Penalty, list[InstanceRepr]]]) -> Any:
        model, (penalty, instances) = chunk
        return (
            model_get_ordering_key(model),
            len(instances),
            len(model_get_name(model)),
        )

    subject_name, subject_penalty, best_instances = ResponseC.decode_from_memory(response_bytes)

    by_model: dict[model.Model, tuple[Penalty, list[InstanceRepr]]] = {}
    for this_model, inst_penalty, instance in best_instances:
        penalty_instances_so_far = by_model.get(this_model)
        if penalty_instances_so_far:
            penalty_so_far, instances_so_far = penalty_instances_so_far
            assert penalty_so_far == inst_penalty, f'assertion failed: {inst_penalty} ≠ {penalty_so_far}'
            instances_so_far.append(instance)
        else:
            by_model[this_model] = (inst_penalty, [instance])

    return Subject(
        name=subject_name,
        penalty=subject_penalty,
        best_models=[
            (model, penalty, instances)
            for model, (penalty, instances)
            in sorted(by_model.items(), key=model_sort_criterion)
        ],
    )

@dataclass
class RenderedGraph:
    # available only if graphviz could be run
    png_url : Optional[str]
    png_bytes : Optional[bytes]

    # available always, for text-based representations
    vertices : list[frozenset[str]]
    edges : list[tuple[frozenset[str], frozenset[str]]]

@dataclass
class RenderedInstance:
    graphviz_missing : bool
    graphs : list[RenderedGraph]
    extra_info : list[tuple[str, str]]

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
            model: model.Model, penalty: Penalty, instances: list[InstanceRepr]
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
            # subject = parent_node.subject
            help_icon = QIcon(platform_specific.get_embedded_file_path('images/qm-16.png'))
            Node.__init__(
                self, parent_node, row,
                fields=(code, Field(icon=help_icon, user_data=code), ''),
                # fields=(code, '', ''),
            )

    class ViewDialog(uic.view_estimated.Ui_ViewEstimated, gui.ExceptionDialog):
        def __init__(self, ds: 'EstimationResult') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            self.alternatives = ds.alternatives
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

        def render_instance(self, instance_code : str) -> RenderedInstance:
            with Core() as core:
                response : InstVizResponse = core.call(
                    'instviz',
                    InstVizRequestC,
                    InstVizResponseC,
                    InstVizRequest(instance_code=instance_code),
                )

            alts = self.alternatives
            def vstr(xs : frozenset[int]) -> str:
                return '"' + ', '.join(sorted(alts[i] for i in xs)) + '"'
            def vset(xs : frozenset[int]) -> frozenset[str]:
                return frozenset(alts[i] for i in xs)

            graphs : list[RenderedGraph] = []
            graphviz_missing = False
            for graph in response.graphs:
                dot_src = (
                    'digraph G {\n bgcolor="transparent" \n'
                    + ''.join(f'{vstr(vs)};\n' for vs in graph.vertices)
                    + ''.join(
                            f'{vstr(greater)} -> {vstr(lesser)};\n'
                            for lesser, greater in graph.edges
                        )
                    + '}'
                )

                try:
                    dot_exe = platform_specific.get_embedded_file_path(
                        'dot.exe',  # deployment Windows
                        'dot',      # deployment elsewhere (?)
                        '/usr/bin/dot',  # dev
                    )
                except platform_specific.FileNotFound:
                    png_bytes = None
                    png_url = None
                    graphviz_missing = True
                else:
                    dot = subprocess.run(
                        [dot_exe, '-Tpng'],
                        capture_output=True,
                        input=dot_src.encode('ascii'),
                    )

                    png_bytes = dot.stdout
                    png_url = 'data:image/png;base64,' + base64.b64encode(png_bytes).decode('ascii')

                graphs.append(RenderedGraph(
                    png_url=png_url,
                    png_bytes=png_bytes,
                    vertices=[vset(xs) for xs in graph.vertices],
                    edges=[(vset(xs), vset(ys)) for xs, ys in graph.edges],
                ))

            return RenderedInstance(
                graphviz_missing=graphviz_missing,
                graphs=graphs,
                extra_info=response.extra_info,
            )

        def dlg_item_clicked(self, idx):
            instance_code = cast(str, self.model.data(idx, Qt.UserRole))
            if instance_code:
                info = self.render_instance(instance_code)
                html = ''

                if info.graphviz_missing:
                    html += '(please install GraphViz to visualise graphs)<br>\n'
                    def vset(xs : frozenset[str]) -> str:
                        return '{' + ','.join(sorted(xs)) + '}'
                    for graph in info.graphs:
                        html += ''.join(
                            f'{vset(greater)} ≥ {vset(lesser)}<br>\n'
                            for lesser, greater in graph.edges
                        )
                        html += '<hr>\n'
                else:
                    for graph in info.graphs:
                        assert graph.png_url
                        html += f'<img src="{graph.png_url}">'
                    html += '<br>\n'

                if info.extra_info:
                    html += ''.join(f'<br>\n{key}: {val}' for key, val in info.extra_info)

                # seems to disappear too quickly on windows
                #
                #QToolTip.showText(QCursor.pos(), html)

                # shows an information icon, which disrupts the message
                #
                #QMessageBox.information(
                #    self,
                #    f'Instance information: {instance_code}',
                #    html,
                #)

                mb = QMessageBox()
                mb.setStandardButtons(
                    QMessageBox.Close
                    if info.graphviz_missing else
                    QMessageBox.Save | QMessageBox.Close
                )
                mb.setWindowTitle(f'Instance information: {instance_code}')
                mb.setText(html)
                btn = mb.exec()

                if btn == QMessageBox.Save:
                    if len(info.graphs) != 1:
                        raise Exception('Saving multiple graphs is not supported yet.')

                    assert info.graphs[0].png_bytes  # button disabled otherwise
                    fname, _ = QFileDialog.getSaveFileName(
                        self,
                        "Save instance visualisation",
                        f'{instance_code.strip("=")}.png',
                        filter="PNG files (*.png)",
                    )
                    if fname:
                        with open(fname, 'wb') as f:
                            f.write(info.graphs[0].png_bytes)

    def __init__(self, name: str, alternatives: Sequence[str]) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects: list[PackedResponse] = []

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

    def export_detailed(self) -> Iterator[Optional[tuple[str,Optional[int|float],int|float,str,str]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            for model, penalty, instances in subject.best_models:
                for instance in sorted(instances):
                    yield (
                        subject.name,
                        from_fraction(penalty.lower_bound)
                            if penalty.lower_bound == penalty.upper_bound
                            else None,
                        from_fraction(penalty.upper_bound),
                        model_get_name(model),
                        base64.b64encode(instance).decode('ascii')
                    )

            yield None  # bump progress

    def export_compact(self) -> Iterator[Optional[tuple[Optional[str],str|int|float,str,int]]]:
        for subject in map(subject_from_response_bytes, self.subjects):
            subject_name: Optional[str] = subject.name
            for model, model_penalty, instances in subject.best_models:
                yield (subject_name, model_penalty.to_csv(), model_get_name(model), len(instances))
                subject_name = None  # don't repeat these

            yield None  # bump progress

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    @classmethod
    def get_codec_progress(_cls) -> CodecProgress['EstimationResult']:
        subjects_encode : Callable[[Worker, FileOut, list[PackedResponse]], None]
        subjects_decode : Callable[[Worker, FileIn], list[PackedResponse]]

        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_size, subjects_encode, subjects_decode = listCP(oneCP(PackedResponseC)).enc_dec()
        intC_encode, intC_decode = intC.enc_dec()

        def get_size(x : 'EstimationResult') -> int:
            return subjects_size(x.subjects)

        def encode(worker : Worker, f : FileOut, x : 'EstimationResult') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'EstimationResult':
            ds = EstimationResult(*DatasetHeaderC_decode(f))
            ds.subjects = subjects_decode(worker, f)
            return ds

        return CodecProgress(get_size, encode, decode)
