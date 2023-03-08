import struct
import logging
import typing
import dataclasses
import numpy as np
from enum import Enum
from io import BytesIO
from fractions import Fraction
from dataclasses import dataclass
from typing import Any, BinaryIO, NewType, NamedTuple, \
    Tuple, Callable, TypeVar, Optional, Dict, Sequence, \
    Generic, cast

log = logging.getLogger(__name__)

T = TypeVar('T')
E = TypeVar('E')
F = TypeVar('F')
K = TypeVar('K')
V = TypeVar('V')

class CodecError(Exception):
    pass

class EOF(CodecError):
    pass

FileIn = NewType('FileIn', BinaryIO)
FileOut = NewType('FileOut', BinaryIO)

@dataclass
class Codec(Generic[T]):
    encode : Callable[[FileOut, T], None]
    decode : Callable[[FileIn], T]

    def enc_dec(self) -> tuple[
        Callable[[FileOut, T], None],
        Callable[[FileIn], T],
    ]:
        return self.encode, self.decode

    def dbg_encode(self, f : FileOut, x : T) -> None:
        bs = self.encode_to_memory(x)
        log.debug('encoding %s: %r' % (self.__class__.__name__, bs))
        f.write(bs)

    def encode_to_memory(self, x : T) -> bytes:
        buf = BytesIO()
        self.encode(typing.cast(FileOut, buf), x)
        return buf.getvalue()

    def decode_from_memory(self, bs : bytes) -> T:
        return self.decode(typing.cast(FileIn, BytesIO(bs)))

    def test(self, x : T) -> None:
        assert x == self.decode_from_memory(self.encode_to_memory(x))

def _intC() -> Codec[int]:
    def encode(f : FileOut, x : int) -> None:
        if x < 0:
            raise CodecError('invalid int: {0}'.format(x))

        # special-casing (x < 0x80)
        # does not seem to help at all

        # this is ugly but it looks like the fastest conversion
        # from int to a single-byte array
        f_write = f.write
        bs = bytearray(1)

        while x >= 0x80:
            bs[0] = 0x80 | (x & 0x7F)
            f_write(bs)
            x >>= 7

        bs[0] = x
        f_write(bs)

    def decode(f : FileIn) -> int:
        value = 0
        ofs = 0
        f_read = f.read

        try:
            while True:
                octet = f_read(1)[0]

                value |= (octet & 0x7F) << ofs
                ofs += 7

                if octet < 0x80:
                    break

            return value
        except IndexError:
            raise EOF()

    return Codec(encode, decode)

intC = _intC()

def _floatC() -> Codec[float]:
    pack, unpack = struct.pack, struct.unpack

    def encode(f : FileOut, x : float) -> None:
        f.write(pack('f', x))

    def decode(f : FileIn) -> float:
        return cast(float, unpack('f', f.read(4))[0])

    return Codec(encode, decode)

floatC = _floatC()

def _doubleC() -> Codec[float]:
    pack, unpack = struct.pack, struct.unpack

    def encode(f : FileOut, x : float) -> None:
        f.write(pack('d', x))

    def decode(f : FileIn) -> float:
        return cast(float, unpack('d', f.read(8))[0])

    return Codec(encode, decode)

doubleC = _doubleC()

def _bytesC() -> Codec[bytes]:
    intC_encode, intC_decode = intC.enc_dec()

    def encode(f : FileOut, x : bytes) -> None:
        intC_encode(f, len(x))
        f.write(x)

    def decode(f : FileIn) -> bytes:
        length = intC_decode(f)
        return f.read(length)

    return Codec(encode, decode)

bytesC = _bytesC()

def _strC() -> Codec[str]:
    bytesC_encode, bytesC_decode = bytesC.enc_dec()

    def encode(f : FileOut, x : str) -> None:
        bytesC_encode(f, x.encode('utf8'))

    def decode(f : FileIn) -> str:
        return bytesC_decode(f).decode('utf8')

    return Codec(encode, decode)

strC = _strC()

def tupleC(*codecs : Codec) -> Codec[tuple]:
    encodes = [c.encode for c in codecs]
    decodes = [c.decode for c in codecs]

    def encode(f : FileOut, xs : tuple) -> None:
        if len(encodes) != len(xs):
            raise CodecError('tuple length mismatch')

        for encode, x in zip(encodes, xs):
            encode(f, x)

    def decode(f : FileIn) -> tuple:
        return tuple(decode(f) for decode in decodes)

    return Codec(encode, decode)

NT = TypeVar('NT', bound=NamedTuple)

def namedtupleC(cls : type[NT], *codecs : Codec) -> Codec[NT]:
    encodes = [c.encode for c in codecs]
    decodes = [c.decode for c in codecs]

    if len(codecs) != len(cls._fields):
        raise CodecError('namedtupleC: %d codecs provided for tuple %s' % (
            len(codecs),
            cls._fields,
        ))

    def encode(f : FileOut, xs : NT) -> None:
        if len(encodes) != len(xs):
            raise CodecError('tuple length mismatch')

        for encode, x in zip(encodes, xs):
            encode(f, x)

    def decode(f : FileIn) -> NT:
        return cls(*[decode(f) for decode in decodes])

    return Codec(encode, decode)


# there are a couple of type: ignore comments here
# because DC should be TypeVar('DC', bound=DataclassInstance),
# according to mypy's error message,
# but i cannot figure out how on earth we're supposed to import DataclassInstance.
#
# let's ignore this for now

DC = TypeVar('DC')
def dataclassC(cls : type[DC], *codecs : Codec) -> Codec[DC]:
    encodes = [c.encode for c in codecs]
    decodes = [c.decode for c in codecs]

    if len(codecs) != len(dataclasses.fields(cls)):  # type: ignore
        raise CodecError('dataclassC: %d codecs provided for dataclass %s' % (
            len(codecs),
            cls,
        ))

    def encode(f : FileOut, xs : DC) -> None:
        xs_tuple = dataclasses.astuple(xs)  # type: ignore
        if len(encodes) != len(xs_tuple):
            raise CodecError('tuple length mismatch')

        for encode, x in zip(encodes, xs_tuple):
            encode(f, x)

    def decode(f : FileIn) -> DC:
        return cast(DC, cls(*[decode(f) for decode in decodes]))  # type: ignore

    return Codec(encode, decode)

def listC(codec : Codec[E]) -> Codec[list[E]]:
    codec_encode, codec_decode = codec.enc_dec()
    intC_encode, intC_decode = intC.enc_dec()

    def encode(f : FileOut, xs : list[E]) -> None:
        intC_encode(f, len(xs))
        for item in xs:
            codec_encode(f, item)

    def decode(f : FileIn) -> list[E]:
        length = intC_decode(f)
        return [codec_decode(f) for _ in range(length)]

    return Codec(encode, decode)

def dictC(k : Codec[K], v : Codec[V]) -> Codec[dict[K,V]]:
    _encode, _decode = listC(tupleC(k, v)).enc_dec()

    def encode(f : FileOut, x : dict[K,V]) -> None:
        _encode(f, cast(list[tuple], x.items()))

    def decode(f : FileIn) -> dict[K,V]:
        return dict(cast(list[tuple[K,V]], _decode(f)))

    return Codec(encode, decode)

def setC(codec : Codec[E]) -> Codec[set[E]]:
    _encode, _decode = listC(codec).enc_dec()

    def decode(f : FileIn) -> set[E]:
        return set(_decode(f))

    return Codec(_encode, decode)  # type: ignore

def frozensetC(codec : Codec[E]) -> Codec[frozenset[E]]:
    _encode, _decode = listC(codec).enc_dec()

    def decode(f : FileIn) -> frozenset[E]:
        return frozenset(_decode(f))

    return Codec(_encode, decode)  # type: ignore

EnumTy = TypeVar('EnumTy', bound=Enum)
def pyEnumC(cls : type[EnumTy], valC : Codec) -> Codec[EnumTy]:
    _encode, _decode = valC.enc_dec()

    def encode(f : FileOut, x : EnumTy) -> None:
        _encode(f, x.value)

    def decode(f : FileIn) -> EnumTy:
        return cls(Enum(_decode(f)))

    return Codec(encode, decode)

def enumC(name : str, alts : Dict[type, Tuple[Codec, ...]]) -> Codec:
    codecs_enc_get = {
        ty._field_defaults['tag']: tupleC(*codecs).encode  # type: ignore
        for ty, codecs in alts.items()
    }.get

    codecs_dec_get = {
        ty._field_defaults['tag']: (ty, tupleC(*codecs).decode)  # type: ignore
        for ty, codecs in alts.items()
    }.get

    intC_encode, intC_decode = intC.enc_dec()

    def encode(f : FileOut, x : tuple) -> None:
        *values, tag = x
        enc = codecs_enc_get(tag)
        if enc is None:
            raise CodecError(f'cannot encode enum tag: {name}/{tag}')

        intC_encode(f, tag)
        enc(f, cast(tuple, values))

    def decode(f : FileIn) -> Any:
        tag = intC_decode(f)
        ty, dec = codecs_dec_get(tag, (None, None))
        if ty is None or dec is None:
            raise CodecError(f'cannot decode enum tag: {name}/{tag}')

        return ty(*dec(f), tag)

    return Codec(encode, decode)

def enum_by_typenameC(name : str, alts : Sequence[Tuple[type, Codec]]) -> Codec:
    codecs_enc_get = {
        ty.__name__: codec.encode
        for ty, codec in alts
    }.get

    codecs_dec_get = {
        ty.__name__: codec.decode
        for ty, codec in alts
    }.get

    strC_encode, strC_decode = strC.enc_dec()

    def encode(f : FileOut, x : tuple) -> None:
        ty = type(x).__name__
        enc = codecs_enc_get(ty)
        if enc is None:
            raise CodecError(f'cannot encode enum class: {name}/{ty}')

        strC_encode(f, ty)
        enc(f, x)

    def decode(f : FileIn) -> Any:
        ty = strC_decode(f)
        dec = codecs_dec_get(ty)
        if dec is None:
            raise CodecError(f'cannot decode enum class: {name}/{ty}')

        return dec(f)

    return Codec(encode, decode)

def _noneC() -> Codec[None]:
    def encode(_f : FileOut, _x : None) -> None:
        pass

    def decode(_f : FileIn) -> None:
        return None

    return Codec(encode, decode)

noneC = _noneC()

def _boolC() -> Codec[bool]:
    intC_encode, intC_decode = intC.enc_dec()

    def encode(f : FileOut, x : bool) -> None:
        intC_encode(f, int(x))

    def decode(f : FileIn) -> bool:
        tag = intC_decode(f)
        if tag == 0:
            return False
        elif tag == 1:
            return True
        else:
            raise CodecError('invalid bool code: %s' % tag)

    return Codec(encode, decode)

boolC = _boolC()

def maybe(codec : Codec[E]) -> Codec[Optional[E]]:
    enc, dec = codec.enc_dec()
    boolC_encode, boolC_decode = boolC.enc_dec()

    def encode(f : FileOut, x : Optional[Any]) -> None:
        if x is None:
            boolC_encode(f, False)
        else:
            boolC_encode(f, True)
            enc(f, x)

    def decode(f : FileIn) -> Optional[Any]:
        if boolC_decode(f):
            return dec(f)
        else:
            return None

    return Codec(encode, decode)

def newtypeC(codec : Codec[E], ctor : Callable[[E], F], proj : Callable[[F], E]) -> Codec[F]:
    enc, dec = codec.enc_dec()

    def encode(f : FileOut, x : F) -> None:
        enc(f, proj(x))

    def decode(f : FileIn) -> F:
        return ctor(dec(f))

    return Codec(encode, decode)

def numpyC(dtype : type) -> Codec[np.ndarray]:
    bytesC_enc, bytesC_dec = bytesC.enc_dec()
    l_enc, l_dec = listC(intC).enc_dec()

    def encode(f : FileOut, x : np.ndarray) -> None:
        assert x.dtype == dtype, f"expected array type: {dtype}, received: {x.dtype}"
        l_enc(f, cast(list[int], x.shape))
        bytesC_enc(f, x.tobytes())

    def decode(f : FileIn) -> np.ndarray:
        shape = tuple(l_dec(f))
        stuff = bytesC_dec(f)
        return np.reshape(
            np.fromstring(stuff, dtype=dtype),  # type:ignore
            newshape=shape,
        )

    return Codec(encode, decode)

def _fractionC() -> Codec[Fraction]:
    intC_enc, intC_dec = intC.enc_dec()

    def encode(f : FileOut, x : Fraction) -> None:
        intC_enc(f, x.numerator)
        intC_enc(f, x.denominator)

    def decode(f : FileIn) -> Fraction:
        return Fraction(
            numerator=intC_dec(f),
            denominator=intC_dec(f),
        )

    return Codec(encode, decode)

fractionC = _fractionC()
