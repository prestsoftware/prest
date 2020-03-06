import openpyxl

import csv
import logging
from typing import Sequence, Any, List, Type, NamedTuple, Callable, Iterator, \
    Optional, FrozenSet, Iterable, NewType, Union

from PyQt5.QtWidgets import QDialog, QMessageBox

import core
import enum
import model
import branding
from gui.progress import Worker, Cancelled
from util.codec import Codec, tupleC, strC, listC, namedtupleC, frozensetC, \
    intC, maybe, bytesC
from util.codec_progress import CodecProgress

log = logging.getLogger(__name__)

def load_raw_csv(fname):
    with open(fname) as f:
        return list(csv.reader(line.strip() for line in f))
    
class ExportVariant(NamedTuple):
    name : str
    column_names : Sequence[str]
    size : int
    get_rows : Callable[[], Iterator[Optional[tuple]]]  # None -> bump progress

class MessageBoxType(enum.Enum):
    INFORMATION = 'info'
    WARNING = 'warn'
    CRITICAL = 'critical'

class ShowMessageBox(NamedTuple):
    type : MessageBoxType
    title : str
    message : str

AnalysisResult = Union[None, ShowMessageBox, 'Dataset']

class Analysis(NamedTuple):
    name : str
    config : Optional[Callable[[], Optional[Any]]]  # display config dialog, return config | can be None
    run :  Callable[[Worker, Any], AnalysisResult]  # (worker, config) -> result
    is_hidden : bool = False

AltSet = FrozenSet[int]
AltSetC = frozensetC(intC)

Menu = AltSet
MenuC = AltSetC

class ChoiceRow(NamedTuple):
    menu : Menu
    default : Optional[int]
    choice : AltSet

ChoiceRowC = namedtupleC(ChoiceRow, MenuC, maybe(intC), AltSetC)

PackedSubject = NewType('PackedSubject', bytes)
PackedSubjectC = bytesC

class Subject(NamedTuple):
    name : str
    alternatives : List[str]
    choices : List[ChoiceRow]

    def csv_set(self, alt_set: Iterable[int]) -> str:
        return ','.join(self.alternatives[i] for i in sorted(alt_set))

    def csv_alt(self, index : Optional[int]) -> Optional[str]:
        if index is None:
            return None
        else:
            return self.alternatives[index]

    def pack(self) -> PackedSubject:
        return PackedSubject(SubjectC.encode_to_memory(self))

    @staticmethod
    def unpack(packed : PackedSubject) -> 'Subject':
        return SubjectC.decode_from_memory(packed)

SubjectC = namedtupleC(Subject, strC, listC(strC), listC(ChoiceRowC))

DatasetHeaderC = tupleC(strC, listC(strC))

class Dataset:
    ViewDialog : Any  # to be overridden in subclasses

    def __init__(self, name: str, alternatives: Sequence[str]) -> None:
        self.name = name
        self.alternatives = list(alternatives)

    def __str__(self):
        return self.label_name()

    def label_type(self):
        return self.__class__.__name__

    def label_name(self):
        return self.name

    def label_alts(self):
        return str(len(self.alternatives))

    def dlg_view(self, _flag=None):
        dlg = self.ViewDialog(self)
        dlg.exec_()

    def label_size(self):
        raise NotImplementedError()

    def get_analyses(self) -> Sequence[Analysis]:
        raise NotImplementedError()

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        raise NotImplementedError()

    def analyse(self, analysis : Analysis) -> Optional['Dataset']:
        if analysis.config is not None:
            config = analysis.config()
            if config is None:
                return None  # dialog cancelled
        else:
            config = None  # no config

        class MyWorker(Worker):
            def work(self):
                return analysis.run(self, config)

        try:
            result = MyWorker().run_with_progress(
                None,  # parent widget
                '{0}...'.format(analysis.name),
            )
        except Cancelled:
            log.debug('analysis cancelled: {0}'.format(analysis.name))
            return None

        if isinstance(result, ShowMessageBox):
            if result.type is MessageBoxType.INFORMATION:
                QMessageBox.information(None, result.title, result.message)
            elif result.type is MessageBoxType.WARNING:
                QMessageBox.warning(None, result.title, result.message)
            elif result.type is MessageBoxType.CRITICAL:
                QMessageBox.critical(None, result.title, result.message)
            else:
                raise Exception('unknown message box type: %s', result.type)

            return None

        return result

    def get_export_variants(self) -> Sequence[ExportVariant]:
        raise NotImplementedError()

    # for testing
    def _get_export_variant(self, name : str) -> ExportVariant:
        for variant in self.get_export_variants():
            if variant.name.lower() == name.lower():
                return variant

        raise ValueError('no such export variant: %s' % name)

    def export(self, fname: str, fformat: str, variant: ExportVariant, worker : Worker) -> None:
        worker.set_work_size(variant.size)
        position = 0

        if '*.csv' in fformat:
            with open(fname, 'w') as f:
                # Python's CSV module and line endings is a mess.
                #
                # Opening the file in binary mode doesn't work (writerow() fails).
                # In text mode, you get \r\r\n on Windows.
                #
                # Hence we force the line terminator here to be '\n', on all platforms
                # and leave the line ending translation to the underlying /file/ layer.
                #
                w = csv.writer(f, quoting=csv.QUOTE_ALL, lineterminator='\n')
                w.writerow(variant.column_names)
                for row in variant.get_rows():
                    if row:
                        w.writerow(row)
                    else:
                        # progress
                        position += 1
                        worker.set_progress(position)

        elif '*.xlsx' in fformat:
            wb = openpyxl.Workbook() 
            wb.properties.creator = branding.PREST_VERSION
            ws = wb.active

            ws.append(variant.column_names)
            for row in variant.get_rows():
                if row:
                    ws.append(row)
                else:
                    # progress
                    position += 1
                    worker.set_progress(position)

            # autosize columns
            # who knows what the units are but it approximately fits
            # furthermore, we fudge the numbers by 1 unit because that looks better
            for column_cells in ws.columns:
                length = max(
                    (len(str(cell.value or '')) for cell in column_cells),
                    default=5
                ) + 1

                if length < 4:
                    length = 4

                ws.column_dimensions[column_cells[0].column].width = length

            wb.save(fname)

        else:
            raise Exception('unknown file export format: %s' % fformat)
