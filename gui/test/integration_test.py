import os
import pytest
from typing import Optional, Tuple

from gui.progress import MockWorker
from model import preorder, unattractive, UndominatedChoice, PartiallyDominantChoice, \
    StatusQuoUndominatedChoice, Overload, PreorderParams
from dataset import load_raw_csv, SubjectC
from gui.estimation import DistanceScore
from gui.estimation import Options as EstimationOpts
from dataset.experimental_data import ExperimentalData
import dataset.budgetary

OVERWRITE_EXPECTED_FILES = False
#OVERWRITE_EXPECTED_FILES = True

def csv_columns(headers, rows):
    columns = zip(*rows)
    return dict(zip(headers, columns))

def check_export(tmpdir, ds, variant_name, csv_fname):
    if OVERWRITE_EXPECTED_FILES:
        out_fname = csv_fname
    else:
        out_fname = os.path.join(tmpdir, '%s.csv' % variant_name)

    variant = ds._get_export_variant(variant_name)
    assert variant
    ds.export(out_fname, '*.csv', variant, MockWorker())

    csv = load_raw_csv(csv_fname)
    expected_cols = csv_columns(csv[0], csv[1:])

    csv = load_raw_csv(out_fname)
    cols = csv_columns(csv[0], csv[1:])

    keys_expected = set(expected_cols.keys())
    keys = set(cols.keys())
    common_keys = keys_expected & keys
    all_keys = keys_expected | keys
    assert 2*len(common_keys) >= len(all_keys)  # at least 1/2 common keys

    # run with pytest -s instead
    #print(common_keys)

    for key in ['subject'] + sorted(common_keys):  # sort to make error locations deterministic
        assert len(expected_cols[key]) == len(cols[key]), key
        assert expected_cols[key] == cols[key], key


DATASETS = (
    ('estimation-models-defaults', 'w x y z', 4),
    ('estimation-models-no-defaults', 'w x y z', 11),
    ('general-hybrid', 'a b c d', 2),
    ('general-merging', 'A B C D E', 1),
)

DATASETS_LONG = (
    ('general-defaults-128', 'A B C D E', 128),
    ('general-no-defaults-128', 'A B C D E', 128),
)

@pytest.mark.parametrize('name,alts,subj_count', DATASETS)
def test_consistency_analysis(tmpdir, name, alts, subj_count):
    rows = load_raw_csv('docs/src/_static/examples/%s.csv' % name)
    ds = ExperimentalData.from_csv('dataset', rows[1:], (0, 1, None, 2))

    assert ds.alternatives == alts.split()
    assert len(ds.subjects) == subj_count

    dsc = ds.analysis_consistency_deterministic(MockWorker(), None)
    assert len(dsc.subjects) == len(ds.subjects)

    check_export(tmpdir, dsc, 'summary', 'gui/test/expected/%s-cons-summary.csv' % name)
    check_export(tmpdir, dsc, 'congruence violations (wide)', 'gui/test/expected/%s-cons-garp.csv' % name)
    check_export(tmpdir, dsc, 'strict general cycles (wide)', 'gui/test/expected/%s-cons-sarp.csv' % name)
    check_export(tmpdir, dsc, 'strict binary cycles (wide)', 'gui/test/expected/%s-cons-sarp-bin.csv' % name)
    check_export(tmpdir, dsc, 'binary cycles (wide)', 'gui/test/expected/%s-cons-garp-bin.csv' % name)

    dst_menus = ds.analysis_tuple_intrans_menus(MockWorker(), None)
    dst_alts = ds.analysis_tuple_intrans_alts(MockWorker(), None)
    assert len(dst_menus.subjects) == len(ds.subjects)
    assert len(dst_alts.subjects) == len(ds.subjects)

    # TODO: figure out the non-determinism of ordering here
    #check_export(tmpdir, dst_menus, 'detailed', 'gui/test/expected/%s-tuple-intrans-detailed-menus.csv' % name)
    #check_export(tmpdir, dst_menus, 'detailed', 'gui/test/expected/%s-tuple-intrans-detailed-alts.csv' % name)

@pytest.mark.long
@pytest.mark.parametrize('name,alts,subj_count', DATASETS_LONG)
def test_consistency_analysis_long(tmpdir, name, alts, subj_count):
    test_consistency_analysis(tmpdir, name, alts, subj_count)

@pytest.mark.parametrize('name,alts,subj_count', DATASETS)
def test_model_estimation(tmpdir, name, alts, subj_count):
    indices : Tuple[Optional[int], ...]
    if name in ('status-quo',):
        indices = (0,1,2,3)
    else:
        indices = (0,1,None,2)

    rows = load_raw_csv('docs/src/_static/examples/%s.csv' % name)
    ds = ExperimentalData.from_csv('aug', rows[1:], indices)

    models = [
        preorder(strict=True, total=True), preorder(strict=False, total=True),
        unattractive(strict=True, total=True), unattractive(strict=False, total=True),
        preorder(strict=True, total=False), preorder(strict=False, total=False),
        UndominatedChoice(strict=True), UndominatedChoice(strict=False),
        PartiallyDominantChoice(fc=True), PartiallyDominantChoice(fc=False),
        Overload(PreorderParams(strict=True, total=True)), Overload(PreorderParams(strict=False, total=True)),
    ]

    if all(cr.default is not None for subj in map(SubjectC.decode_from_memory, ds.subjects) for cr in subj.choices):
        models.append(StatusQuoUndominatedChoice())

    dsm = ds.analysis_estimation(
        MockWorker(),
        EstimationOpts(
            models,
            disable_parallelism=False,
            disregard_deferrals=False,
            distance_score=DistanceScore.HOUTMAN_MAKS
        ),
    )

    check_export(tmpdir, dsm, 'compact (human-friendly)', 'gui/test/expected/%s-models-compact.csv' % name)
    check_export(tmpdir, dsm, 'detailed (machine-friendly)', 'gui/test/expected/%s-models-detailed.csv' % name)

@pytest.mark.long
@pytest.mark.parametrize('name,alts,subj_count', DATASETS_LONG)
def test_model_estimation_long(tmpdir, name, alts, subj_count):
    test_model_estimation(tmpdir, name, alts, subj_count)

def test_budgetary(tmpdir):
    ds = dataset.budgetary.load_from_csv('docs/src/_static/examples/budgetary.csv')
    newds = ds.analysis_consistency(MockWorker(), None)

    check_export(tmpdir, newds, 'Summary', 'gui/test/expected/budgetary-summary.csv')
    check_export(tmpdir, newds, 'Violations by cycle length', 'gui/test/expected/budgetary-cycles.csv')

def test_integrity(tmpdir):
    rows = load_raw_csv('docs/src/_static/examples/integrity.csv')
    ds = ExperimentalData.from_csv('dataset', rows[1:], (0, 1, None, 2))
    nds = ds.analysis_integrity_check(MockWorker(), None)
    assert isinstance(nds, dataset.integrity_check.IntegrityCheck)

    assert len(nds.subjects) == 1
    assert nds.subjects[0].name == 'a'
    assert nds.subjects[0].issues == [
        dataset.integrity_check.RepeatedMenu(
            menu={0,1},
        ),
        dataset.integrity_check.ChoiceNotInMenu(
            menu={0,1},
            choice=2,
        ),
    ]

if __name__ == '__main__':
    import logging
    logging.basicConfig(level=logging.DEBUG)
    #test_stats()
