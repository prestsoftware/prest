from gui.progress import MockWorker
from dataset.experimental_data import ExperimentalData

def test_merging():
    rows = [
        'subjA Ca,Hi,Pa Ca',
        'subjA Ca,Pa Pa',
        'subjA Ca,Hi,Pa Pa',
        'subjA Ca Ca',
        'subjA Ca,Hi,Pa ',
        'subjA Ca,Pa Ca',
        'subjA Ca,Pa,Hi Hi',
    ]
    ds = ExperimentalData.from_csv('X', [r.split(' ') for r in rows], (0,1,None,2))
    newds = ds.analysis_merge_choices(
        MockWorker(),
        ExperimentalData.MergeOptions(
            track_deferrals_separately=True,
        ),
    )

    assert list(newds.export_detailed()) == [
        ('subjA', 'Ca,Hi,Pa', None, 'Ca,Hi,Pa'),
        ('subjA', 'Ca,Pa', None, 'Ca,Pa'),
        ('subjA', 'Ca', None, 'Ca'),
        ('subjA', 'Ca,Hi,Pa', None, ''),
        None,  # bump progress
    ]

def test_merging_default():
    rows = [
        'subjA Ca,Hi,Pa Pa Ca',
        'subjA Ca,Pa Pa Pa',
        'subjA Ca,Hi,Pa Pa Pa',
        'subjA Ca Ca Ca',
        'subjA Ca,Hi,Pa Ca ',
        'subjA Ca,Pa Pa Ca',
        'subjA Ca,Pa,Hi Ca Hi',
        'subjA Ca,Hi,Pa Hi ',
    ]
    ds = ExperimentalData.from_csv('X', [r.split(' ') for r in rows], (0,1,2,3))
    newds = ds.analysis_merge_choices(
        MockWorker(),
        ExperimentalData.MergeOptions(
            track_deferrals_separately=True,
        ),
    )

    assert list(newds.export_detailed()) == [
        ('subjA', 'Ca,Hi,Pa', 'Pa', 'Ca,Pa'),
        ('subjA', 'Ca,Pa', 'Pa', 'Ca,Pa'),
        ('subjA', 'Ca', 'Ca', 'Ca'),
        ('subjA', 'Ca,Hi,Pa', 'Ca', ''),
        ('subjA', 'Ca,Hi,Pa', 'Ca', 'Hi'),
        ('subjA', 'Ca,Hi,Pa', 'Hi', ''),
        None,  # bump progress
    ]
