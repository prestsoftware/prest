import os
import pytest
from hypothesis import given
from hypothesis.strategies import integers, lists, text

from util.codec import strC, intC
from util.packed_list import PackedList

ints = integers(min_value=0)

@given(lists(ints))
def test_basic_ints(xs):
    xs_packed = PackedList(intC, xs)
    assert xs == list(xs_packed)

@given(lists(text()))
def test_basic_strings(xs):
    xs_packed = PackedList(strC, xs)
    assert xs == list(xs_packed)
