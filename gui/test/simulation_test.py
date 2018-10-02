import io
import pytest

import dataset
import simulation
from core import Core

def test_simulation(nsubjects=256, f_mock=None):
    with Core(f_mock=f_mock) as core:
        response = simulation.run(core, simulation.Request(
            name='random',
            alternatives=('A','B','C','D','E'),
            gen_menus=simulation.GenMenus(
                generator=simulation.Exhaustive(),
                defaults=False,
            ),
            gen_choices=simulation.Uniform(
                forced_choice=True,
                multiple_choice=False,
            ),
        ))

    assert len(response.subject_packed) == 223

#def test_simulation_gen():
def _simulation_gen():
    f_in = io.BytesIO()
    f_out = io.BytesIO()
    with Core(f_tee=(f_in, f_out)) as core:
        response = simulation.run(core, simulation.Request(
            name='random',
            alternatives=('A','B','C','D','E'),
            gen_menus=simulation.GenMenus(
                generator=simulation.Exhaustive(),
                defaults=False,
            ),
            gen_choices=simulation.Uniform(
                forced_choice=True,
                multiple_choice=False,
            ),
        ))

    with open('in.bin', 'wb') as f:
        f.write(f_in.getbuffer())

    with open('out.bin', 'wb') as f:
        f.write(f_out.getbuffer())

def simulation_mock():
    with open('out.bin', 'rb') as f:
        test_simulation(nsubjects=8192, f_mock=f)

if __name__ == '__main__':
    import cProfile
    cProfile.run('simulation_mock()', sort='cumtime')
