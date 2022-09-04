import doc
import dataset
import util.tree_model
import uic.view_dataset
import platform_specific
from dataset import Dataset, Analysis, ExportVariant, DatasetHeaderC
from typing import Sequence, NamedTuple, List, Iterator, Tuple, cast
from gui.progress import Worker, Cancelled
from util.codec import namedtupleC, strC, intC, listC, tupleC, FileIn, FileOut
from util.codec_progress import CodecProgress, listCP, oneCP

from PyQt5.QtGui import QIcon
from PyQt5.QtWidgets import QDialog, QTreeWidgetItem, QHeaderView

class BoundEstimate(NamedTuple):
    lower : int
    upper : int

    def to_str(self) -> str:
        if self.lower == self.upper:
            return str(self.lower)
        else:
            return f'{self.lower} ≤ x ≤ {self.upper}'

BoundEstimateC = namedtupleC(BoundEstimate, intC, intC)

class Violations(NamedTuple):
    garp : int
    sarp : int

ViolationsC = namedtupleC(Violations, intC, intC)

class Subject(NamedTuple):
    subject_name : str
    violations : List[Tuple[int, Violations]]  # cycle length, counts
    warp_strict : int
    warp_nonstrict : int
    hm_garp : BoundEstimate
    hm_sarp : BoundEstimate
    hm_warp_strict : BoundEstimate
    hm_warp_nonstrict : BoundEstimate

SubjectC = namedtupleC(Subject, strC, listC(tupleC(intC, ViolationsC)), intC, intC,
    BoundEstimateC, BoundEstimateC,
    BoundEstimateC, BoundEstimateC,
)

class ViolationsNode(util.tree_model.Node):
    def __init__(self, parent_node, row, len_viol : Tuple[int, Violations]) -> None:
        cycle_length, violations = len_viol
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(
                '',
                cycle_length,
                violations.garp,
                violations.sarp,
                '-',
                '-',
                '-',
                '-',
                '-',
                '-',
            ),
        )

class SubjectNode(util.tree_model.Node):
    def __init__(self, parent_node, row, subject) -> None:
        self.violations = subject.violations
        util.tree_model.Node.__init__(
            self, parent_node, row,
            fields=(
                subject.subject_name,
                '-',
                sum(v.garp for _,v in subject.violations),
                sum(v.sarp for _,v in subject.violations),
                subject.warp_strict,
                subject.warp_nonstrict,
                subject.hm_garp.to_str(),
                subject.hm_sarp.to_str(),
                subject.hm_warp_strict.to_str(),
                subject.hm_warp_nonstrict.to_str(),
            ),
            child_count=len(subject.violations),
        )

    def create_child(self, row: int) -> ViolationsNode:
        return ViolationsNode(self, row, self.violations[row])

class RootNode(util.tree_model.RootNode):
    def __init__(self, subjects : List[Subject]) -> None:
        util.tree_model.RootNode.__init__(self, len(subjects))
        self.subjects = subjects

    def create_child(self, row: int) -> SubjectNode:
        return SubjectNode(self, row, self.subjects[row])

class BudgetaryConsistency(Dataset):
    class ViewDialog(QDialog, uic.view_dataset.Ui_ViewDataset):
        def __init__(self, ds : 'BudgetaryConsistency') -> None:
            QDialog.__init__(self)
            self.setupUi(self)

            help_icon = QIcon(platform_specific.get_embedded_file_path(
                'images/qm-16.png',      # deploy
                'gui/images/qm-16.png',  # devel
            ))
            F = util.tree_model.Field

            self.ds = ds
            self.model = util.tree_model.TreeModel(
                RootNode(ds.subjects),
                headers=[
                    'Subject',
                    'Cycle length',
                    F('GARP', help_icon,
                        'consistency/cons_budgetary.html#generalized-axiom-of-revealed-preference-garp'),
                    F('SARP', help_icon,
                        'consistency/cons_budgetary.html#strong-axiom-of-revealed-preference-sarp'),
                    F('WARP (strict)', help_icon,
                        'consistency/cons_budgetary.html#weak-axiom-of-revealed-preference-warp'),
                    F('WARP (non-strict)', help_icon,
                        'consistency/cons_budgetary.html#weak-axiom-of-revealed-preference-warp'),
                    F('HM (GARP)', help_icon,
                        'consistency/cons_budgetary.html#houtman-maks-index-hm'),
                    F('HM (SARP)', help_icon,
                        'consistency/cons_budgetary.html#houtman-maks-index-hm'),
                    F('HM (WARP strict)', help_icon,
                        'consistency/cons_budgetary.html#houtman-maks-index-hm'),
                    F('HM (WARP non-strict)', help_icon,
                        'consistency/cons_budgetary.html#houtman-maks-index-hm'),
                ],
            )

            self.twRows.setModel(self.model)

            hdr = self.twRows.header()
            hdr.setSectionResizeMode(QHeaderView.ResizeToContents)
            hdr.setStretchLastSection(False)

            hdr.setSectionsClickable(True)
            hdr.sectionClicked.connect(self.header_clicked)

        def header_clicked(self, idx) -> None:
            name = self.model.headers[idx].user_data
            if not name: return

            doc.open_in_browser(name)

    def __init__(
        self,
        name : str,
        alternatives : Sequence[str],
        subjects : List[Subject] = []
    ) -> None:
        Dataset.__init__(self, name, alternatives)
        self.subjects : List[Subject] = subjects

    def label_size(self) -> str:
        return '%d subjects' % len(self.subjects)

    def get_analyses(self) -> Sequence[Analysis]:
        return []

    @staticmethod
    def get_codec_progress() -> CodecProgress:
        DatasetHeaderC_encode, DatasetHeaderC_decode = DatasetHeaderC.enc_dec()
        subjects_get_size, subjects_encode, subjects_decode = listCP(oneCP(SubjectC)).enc_dec()

        def get_size(x : 'BudgetaryConsistency') -> int:
            return cast(int, subjects_get_size(x.subjects))

        def encode(worker : Worker, f : FileOut, x : 'BudgetaryConsistency') -> None:
            DatasetHeaderC_encode(f, (x.name, x.alternatives))
            subjects_encode(worker, f, x.subjects)

        def decode(worker : Worker, f : FileIn) -> 'BudgetaryConsistency':
            name, alts = DatasetHeaderC_decode(f)
            subjects = subjects_decode(f)
            return BudgetaryConsistency(name, alts, subjects)

        return CodecProgress(get_size, encode, decode)

    def get_export_variants(self) -> Sequence[ExportVariant]:
        return [
            ExportVariant(
                name='Summary',
                column_names=('subject', 'garp', 'sarp', 'warp_strict', 'warp_nonstrict',
                    'hm_garp_lower', 'hm_garp_upper', 'hm_sarp_lower', 'hm_sarp_upper',
                    'hm_warp_strict_lower', 'hm_warp_strict_upper',
                    'hm_warp_nonstrict_lower', 'hm_warp_nonstrict_upper',
                ),
                get_rows=self.export_summary,
                size=len(self.subjects),
            ),
            ExportVariant(
                name='Violations by cycle length',
                column_names=('subject', 'cycle_length', 'garp', 'sarp'),
                get_rows=self.export_breakdown,
                size=len(self.subjects),
            ),
        ]

    def export_summary(self) -> Iterator[Tuple[str, int, int, int, int, int, int, int, int, int, int, int, int]]:
        for subject in self.subjects:
            yield (
                subject.subject_name,
                sum(v.garp for _l, v in subject.violations),
                sum(v.sarp for _l, v in subject.violations),
                subject.warp_strict,
                subject.warp_nonstrict,
                subject.hm_garp.lower,
                subject.hm_garp.upper,
                subject.hm_sarp.lower,
                subject.hm_sarp.upper,
                subject.hm_warp_strict.lower,
                subject.hm_warp_strict.upper,
                subject.hm_warp_nonstrict.lower,
                subject.hm_warp_nonstrict.upper,
            )

    def export_breakdown(self) -> Iterator[Tuple[str, int, int, int]]:
        for subject in self.subjects:
            for cycle_length, violation in subject.violations:
                yield (
                    subject.subject_name,
                    cycle_length,
                    violation.garp,
                    violation.sarp,
                )
