import pytest
from hypothesis import given
from hypothesis.strategies import integers, lists, tuples, text

from util.codec import intC, listC, tupleC, strC, bytesC

ints = integers(min_value=0)

@given(ints)
def test_int(x):
    intC.test(x)

@given(lists(ints))
def test_list_int(xs):
    listC(intC).test(xs)

@given(lists(text()))
def test_list_str(xs):
    listC(strC).test(xs)
    
@given(tuples(lists(text()), ints, lists(tuples(ints, text()))))
def test_complicated(xs):
    tupleC(listC(strC), intC, listC(tupleC(intC, strC))).test(xs)
