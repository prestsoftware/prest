from typing import NamedTuple, Union, List, cast

from core import Core
from dataset import ChoiceRow, ChoiceRowC, Menu, MenuC, Subject, SubjectC, \
    PackedSubject, PackedSubjectC
from util.codec import listC, bytesC, enumC, intC, namedtupleC, boolC, strC

class Exhaustive(NamedTuple):
    tag : int = 0

class SampleWithReplacement(NamedTuple):
    menu_count : int
    tag : int = 1

class Copycat(NamedTuple):
    subject_packed : PackedSubject
    tag : int = 2

class Binary(NamedTuple):
    tag : int = 3

MenuGenerator = Union[
    Exhaustive,
    SampleWithReplacement,
    Copycat,
    Binary,
]

MenuGeneratorC = enumC('GenMenus', {
    Exhaustive: (),
    SampleWithReplacement: (intC,),
    Copycat: (PackedSubjectC,),
    Binary: (),
})

class GenMenus(NamedTuple):
    generator : MenuGenerator
    defaults : bool

GenMenusC = namedtupleC(GenMenus, MenuGeneratorC, boolC)

class Instance(NamedTuple):
    code : bytes
    tag : int = 0

class Uniform(NamedTuple):
    forced_choice : bool
    multiple_choice : bool
    tag : int = 1

GenChoices = Union[
    Instance,
    Uniform,
]

GenChoicesC = enumC('GenChoices', {
    Instance: (bytesC,),
    Uniform: (boolC, boolC),
})

class Request(NamedTuple):
    name : str
    alternatives : List[str]
    gen_menus : GenMenus
    gen_choices : GenChoices
    preserve_deferrals : bool

RequestC = namedtupleC(Request, strC, listC(strC), GenMenusC, GenChoicesC, boolC)

class Response(NamedTuple):
    subject_packed : PackedSubject
    observation_count : int

ResponseC = namedtupleC(Response, PackedSubjectC, intC)

def run(core : Core, request : Request) -> Response:
    return core.call('simulation', RequestC, ResponseC, request)
