from __future__ import annotations

import base64
import logging
import subprocess

from PyQt5.QtWidgets import QMessageBox, QFileDialog

import platform_specific

from core import Core
from gui.progress import Worker
from dataclasses import dataclass
from dataset import Dataset, Analysis, ExportVariant, DatasetHeaderC
from typing import Sequence, NewType, Optional
from util.codec import FileIn, FileOut, dataclassC, bytesC, listC, frozensetC, \
    intC, tupleC, strC
from util.codec_progress import CodecProgress, oneCP

log = logging.getLogger(__name__)

InstanceRepr = NewType('InstanceRepr', bytes)
InstanceReprC = bytesC

@dataclass
class Response:
    instance_repr : InstanceRepr

ResponseC = dataclassC(Response, InstanceReprC)

@dataclass
class InstVizRequest:
    instance_code : str

InstVizRequestC = dataclassC(InstVizRequest, strC)

@dataclass
class GraphRepr:
    vertices : list[frozenset[int]]
    edges : list[tuple[frozenset[int], frozenset[int]]]

GraphReprC = dataclassC(GraphRepr, listC(frozensetC(intC)), listC(tupleC(frozensetC(intC), frozensetC(intC))))

@dataclass
class InstVizResponse:
    graphs : list[GraphRepr]
    extra_info : list[tuple[str, str]]

InstVizResponseC = dataclassC(InstVizResponse, listC(GraphReprC), listC(tupleC(strC, strC)))

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

def instance_repr_to_code(repr : InstanceRepr) -> str:
    return base64.b64encode(repr).decode('ascii')

def instance_code_to_repr(code : str) -> InstanceRepr:
    return InstanceRepr(base64.b64decode(code.encode('ascii')))

def render_instance(alternatives : Sequence[str], instance_code : str) -> RenderedInstance:
    with Core() as core:
        response : InstVizResponse = core.call(
            'instviz',
            InstVizRequestC,
            InstVizResponseC,
            InstVizRequest(instance_code=instance_code),
        )

    def vstr(xs : frozenset[int]) -> str:
        return '"' + ', '.join(sorted(alternatives[i] for i in xs)) + '"'

    def vset(xs : frozenset[int]) -> frozenset[str]:
        return frozenset(alternatives[i] for i in xs)

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


def display_instance(alternatives : Sequence[str], instance_code : str) -> None:
    info = render_instance(alternatives, instance_code)
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
            None,
            "Save instance visualisation",
            f'{instance_code.strip("=")}.png',
            filter="PNG files (*.png)",
        )

        if fname:
            with open(fname, 'wb') as f:
                f.write(info.graphs[0].png_bytes)

class AggregatedPreferences(Dataset):
    def __init__(
        self,
        name : str,
        alternatives : Sequence[str],
        response : Response,
    ) -> None:
        Dataset.__init__(self, name, alternatives)
        self.response = response

    def dlg_view(self, _flag : Optional[bool] = None) -> None:
        display_instance(
            self.alternatives,
            instance_repr_to_code(self.response.instance_repr),
        )

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
