import logging
import collections
import platform_specific
from typing import NamedTuple, Iterator, Tuple, Union, Sequence, Optional, Any

from util.codec import Codec, FileIn, FileOut, namedtupleC, strC, \
        intC, frozensetC, listC, bytesC, tupleC, maybe, \
        enumC, boolC

log = logging.getLogger(__name__)

class PreorderParams(NamedTuple):
    strict : Optional[bool]
    total : Optional[bool]

SIGN = {True: '', False: '¬', None: '?'}
def strPP(p) -> str:
    return '({0}S, {1}T)'.format(
        SIGN[p.strict], SIGN[p.total]
    )

PreorderParamsC = namedtupleC(PreorderParams, maybe(boolC), maybe(boolC))

class PreorderMaximization(NamedTuple):
    p : PreorderParams
    tag : int = 0

class Unattractiveness(NamedTuple):
    p : PreorderParams
    tag : int = 1

class UndominatedChoice(NamedTuple):
    strict : bool
    tag : int = 2

class PartiallyDominantChoice(NamedTuple):
    fc : bool
    tag : int = 3

class StatusQuoUndominatedChoice(NamedTuple):
    tag : int = 4

class Overload(NamedTuple):
    p : PreorderParams
    tag : int = 5

class TopTwo(NamedTuple):
    tag : int = 6

class SequentiallyRationalizableChoice(NamedTuple):
    tag : int = 7

class HybridDomination(NamedTuple):
    strict : bool
    tag : int = 8

Model = Union[
    PreorderMaximization,
    Unattractiveness,
    UndominatedChoice,
    PartiallyDominantChoice,
    StatusQuoUndominatedChoice,
    Overload,
    TopTwo,
    SequentiallyRationalizableChoice,
    HybridDomination,
]

ModelC = enumC('Model', {
    PreorderMaximization: (PreorderParamsC,),
    Unattractiveness: (PreorderParamsC,),
    UndominatedChoice: (boolC,),
    PartiallyDominantChoice: (boolC,),
    StatusQuoUndominatedChoice: (),
    Overload: (PreorderParamsC,),
    TopTwo: (),
    SequentiallyRationalizableChoice: (),
    HybridDomination: (boolC,),
})

# dicts are ordered from python 3.5 onwards
SPECIAL_NAMES = {
    PreorderMaximization(PreorderParams(total=True, strict=True)):
        'Utility Maximization (Strict)',
    PreorderMaximization(PreorderParams(total=True, strict=False)):
        'Utility Maximization (Non-strict)',
    Unattractiveness(PreorderParams(total=True, strict=True)):
        'Utility Maximization with an Outside Option (Strict)',
    Unattractiveness(PreorderParams(total=True, strict=False)):
        'Utility Maximization with an Outside Option (Non-strict)',
    PreorderMaximization(PreorderParams(total=False, strict=True)):
        'Maximally Dominant Choice (Strict)',
    PreorderMaximization(PreorderParams(total=False, strict=False)):
        'Maximally Dominant Choice (Non-strict)',
    UndominatedChoice(strict=True):
        'Undominated Choice (Strict)',
    UndominatedChoice(strict=False):
        'Undominated Choice (Non-strict)',
    PartiallyDominantChoice(fc=True):
        'Partially Dominant Choice (Forced)',
    PartiallyDominantChoice(fc=False):
        'Partially Dominant Choice (Non-forced)',
    StatusQuoUndominatedChoice():
        'Status-Quo-Biased Undominated Choice',
    Overload(PreorderParams(strict=True, total=True)):
        'Overload-Constrained Utility Maximization (Strict)',
    Overload(PreorderParams(strict=False, total=True)):
        'Overload-Constrained Utility Maximization (Non-strict)',
    TopTwo():
        'Top-Two Choice',
    SequentiallyRationalizableChoice():
        'Sequentially Rationalizable Choice',
    HybridDomination(strict=True):
        'Hybrid Dominance and Undomination (strict)',
    HybridDomination(strict=False):
        'Hybrid Dominance and Undomination (non-strict)',
}

ORDERING_INDICES = (
    PreorderMaximization(PreorderParams(total=True, strict=True)),
    PreorderMaximization(PreorderParams(total=True, strict=False)),
    Unattractiveness(PreorderParams(total=True, strict=True)),
    Unattractiveness(PreorderParams(total=True, strict=False)),
)

UPPER_BOUND_MODELS = {
    SequentiallyRationalizableChoice(),
}

# returns something comparable
def get_ordering_key(model : Model) -> Any:
    try:
        return ORDERING_INDICES.index(model)
    except ValueError:
        return 1024   # not mentioned

def get_name(model : Model) -> str:
    name = SPECIAL_NAMES.get(model)
    if name:
        return name

    return str(model)

class Category(NamedTuple):
    name : str
    children : Sequence  # recursive types not supported in mypy

class ModelGroup(NamedTuple):
    name: str
    help_url: Optional[str]
    variants: Sequence[Optional[Tuple[str, Model]]]

def mgroup(name: str, help_url: Optional[str]=None, *variants : Optional[Tuple[str, Model]]) -> ModelGroup:
    return ModelGroup(name, help_url, variants)

def preorder(strict: Optional[bool]=None, total: Optional[bool]=None) -> Model:
    return PreorderMaximization(PreorderParams(strict, total))

def unattractive(strict: Optional[bool]=None, total: Optional[bool]=None) -> Model:
    return Unattractiveness(PreorderParams(strict, total))

def sublabel(main : str, detail : str) -> str:
    return '{0}<br/><small>{1}</small>'.format(main, detail)

MODELS = [
    Category('Choice without a Default Alternative', (
        Category('Forced Choice', (
            mgroup('Utility Maximization',
                'models/fc.html#utility-maximization',
                ('Strict', preorder(strict=True, total=True)),
                ('Non-strict', preorder(strict=False, total=True)),
            ),
            mgroup('Top-Two Choice',
                'models/fc.html#top-two-choice',
                ('Strict', TopTwo()),
                None,
            ),
            mgroup('Incomplete-Preference Maximization: Undominated Choice',
                'models/fc.html#incomplete-preference-maximization-undominated-choice',
                ('Strict', UndominatedChoice(strict=True)),
                ('Non-strict', UndominatedChoice(strict=False)),
            ),
            mgroup('Incomplete-Preference Maximization: Partially Dominant Choice (forced)',
                'models/fc.html#incomplete-preference-maximization-partially-dominant-choice-forced',
                ('Strict', PartiallyDominantChoice(fc=True)),
                None,
            ),
            mgroup('Incomplete-Preference Maximization: Hybrid Dominance & Undomination',
                'models/fc.html#incomplete-preference-maximization-hybrid-dominance-undomination',
                ('Strict', HybridDomination(strict=True)),
                ('Non-strict', HybridDomination(strict=False)),
            ),
            mgroup(sublabel(
                    'Sequentially Rationalizable Choice',
                    '(experimental/partial functionality)',
                ),
                'models/fc.html#sequentially-rationalizable-choice',
                ('Strict', SequentiallyRationalizableChoice()),
                None,
            ),
            # mgroup('Choice with limited attention'),
        )),
        Category('Non-Forced Choice', (
            mgroup('Utility Maximization with an Outside Option',
                'models/nfc.html#utility-maximization-with-an-outside-option',
                ('Strict', unattractive(strict=True, total=True)),
                ('Non-strict', unattractive(strict=False, total=True)),
            ),
            mgroup('Overload-Constrained Utility Maximization',
                'models/nfc.html#overload-constrained-utility-maximization',
                ('Strict', Overload(PreorderParams(strict=True, total=True))),
                ('Non-Strict', Overload(PreorderParams(strict=False, total=True))),
            ),
            mgroup('Incomplete-Preference Maximization: Maximally Dominant Choice',
                'models/nfc.html#incomplete-preference-maximization-maximally-dominant-choice',
                ('Strict', preorder(strict=True, total=False)),
                ('Non-strict', preorder(strict=False, total=False)),
            ),
            mgroup('Incomplete-Preference Maximization: Partially Dominant Choice (non-forced)',
                'models/nfc.html#incomplete-preference-maximization-partially-dominant-choice-non-forced',
                ('Strict', PartiallyDominantChoice(fc=False)),
                None,
            ),
        )),
    )),
    Category('Choice with a Default Alternative', (
        mgroup('Incomplete-Preference Maximization: Status-Quo-Biased Undominated Choice (Bewley)',
            'models/default.html#status-quo-biased-undominated-choice-bewley-model',
            ('Strict', StatusQuoUndominatedChoice()),
            None,
        ),
    )),
]

def names_in_order() -> Iterator[str]:
    def traverse(item):
        if isinstance(item, Category):
            for child in item.children:
                for name in traverse(child):
                    yield name
        else:
            yield item.name

    for item in MODELS:
        for name in traverse(item):
            yield name
