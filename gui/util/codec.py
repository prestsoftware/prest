import json
import struct
import logging
import base64
import typing
from io import BytesIO
import numpy as np
from typing import Union, Any, BinaryIO, Type, NewType, NamedTuple, List, \
    Tuple, Callable, TypeVar, Optional, Dict, Sequence, Generic

log = logging.getLogger(__name__)

T = TypeVar('T')

class CodecError(Exception):
    pass

class EOF(CodecError):
    pass

FileIn = NewType('FileIn', BinaryIO)
FileOut = NewType('FileOut', BinaryIO)

class Codec(NamedTuple):
    encode : Callable[[FileOut, Any], None]
    decode : Callable[[FileIn], Any]

    def dbg_encode(self, f : FileOut, x : Any) -> None:
        bs = self.encode_to_memory(x)
        log.debug('encoding %r: %r' % (self.__class__.__name__, bs))
        f.write(bs)

    def encode_to_memory(self, x : Any) -> bytes:
        buf = BytesIO()
        self.encode(typing.cast(FileOut, buf), x)
        return buf.getvalue()

    def decode_from_memory(self, bs : bytes) -> Any:
        return self.decode(typing.cast(FileIn, BytesIO(bs)))

    def test(self, x : Any) -> None:
        assert x == self.decode_from_memory(self.encode_to_memory(x))

def _intC() -> Codec:
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

def _floatC() -> Codec:
    pack, unpack = struct.pack, struct.unpack

    def encode(f : FileOut, x : float) -> None:
        f.write(pack('f', x))

    def decode(f : FileIn) -> float:
        return unpack('f', f.read(4))[0]

    return Codec(encode, decode)

floatC = _floatC()

def _doubleC() -> Codec:
    pack, unpack = struct.pack, struct.unpack

    def encode(f : FileOut, x : float) -> None:
        f.write(pack('d', x))

    def decode(f : FileIn) -> float:
        return unpack('d', f.read(8))[0]

    return Codec(encode, decode)

doubleC = _doubleC()

def _bytesC() -> Codec:
    intC_encode, intC_decode = intC

    def encode(f : FileOut, x : bytes) -> None:
        intC_encode(f, len(x))
        f.write(x)

    def decode(f : FileIn) -> bytes:
        length = intC_decode(f)
        return f.read(length)

    return Codec(encode, decode)

bytesC = _bytesC()

def _strC() -> Codec:
    bytesC_encode, bytesC_decode = bytesC

    def encode(f : FileOut, x : str) -> None:
        bytesC_encode(f, x.encode('utf8'))

    def decode(f : FileIn) -> str:
        return bytesC_decode(f).decode('utf8')

    return Codec(encode, decode)

strC = _strC()

def tupleC(*codecs : Codec) -> Codec:
    encodes = [enc for enc, _dec in codecs]
    decodes = [dec for _enc, dec in codecs]

    def encode(f : FileOut, xs : tuple) -> None:
        if len(encodes) != len(xs):
            raise CodecError('tuple length mismatch')

        for encode, x in zip(encodes, xs):
            encode(f, x)

    def decode(f : FileIn) -> tuple:
        return tuple(decode(f) for decode in decodes)

    return Codec(encode, decode)

def namedtupleC(cls : Type, *codecs : Codec) -> Codec:
    encodes = [enc for enc, _dec in codecs]
    decodes = [dec for _enc, dec in codecs]

    if len(codecs) != len(cls._fields):
        raise CodecError('namedtupleC: %d codecs provided for tuple %s' % (
            len(codecs),
            cls._fields,
        ))

    def encode(f : FileOut, xs : tuple) -> None:
        if len(encodes) != len(xs):
            raise CodecError('tuple length mismatch')

        for encode, x in zip(encodes, xs):
            encode(f, x)

    def decode(f : FileIn) -> tuple:
        return cls(*[decode(f) for decode in decodes])

    return Codec(encode, decode)

def listC(codec : Codec) -> Codec:
    codec_encode, codec_decode = codec
    intC_encode, intC_decode = intC

    def encode(f : FileOut, xs : list) -> None:
        intC_encode(f, len(xs))
        for item in xs:
            codec_encode(f, item)

    def decode(f : FileIn) -> list:
        length = intC_decode(f)
        return [codec_decode(f) for _ in range(length)]

    return Codec(encode, decode)

def dictC(k : Codec, v : Codec) -> Codec:
    _encode, _decode = listC(tupleC(k, v))

    def encode(f : FileOut, x : dict) -> None:
        _encode(f, x.items())

    def decode(f : FileIn) -> dict:
        return dict(_decode(f))

    return Codec(encode, decode)

def setC(codec : Codec) -> Codec:
    _encode, _decode = listC(codec)

    def decode(f : FileIn) -> set:
        return set(_decode(f))

    return Codec(_encode, decode)

def frozensetC(codec : Codec) -> Codec:
    _encode, _decode = listC(codec)

    def decode(f : FileIn) -> frozenset:
        return frozenset(_decode(f))

    return Codec(_encode, decode)

def enumC(name : str, alts : Dict[type, Tuple[Codec, ...]]) -> Codec:
    codecs_enc_get = {
        ty._field_defaults['tag']: tupleC(*codecs).encode  # type: ignore
        for ty, codecs in alts.items()
    }.get

    codecs_dec_get = {
        ty._field_defaults['tag']: (ty, tupleC(*codecs).decode)  # type: ignore
        for ty, codecs in alts.items()
    }.get

    intC_encode, intC_decode = intC

    def encode(f : FileOut, x : tuple) -> None:
        *values, tag = x
        enc = codecs_enc_get(tag)
        if enc is None:
            raise CodecError(f'cannot encode enum tag: {name}/{tag}')

        intC_encode(f, tag)
        enc(f, values)

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

    strC_encode, strC_decode = strC

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

def _noneC() -> Codec:
    def encode(f : FileOut, x : None) -> None:
        pass

    def decode(f : FileIn) -> None:
        return None

    return Codec(encode, decode)

noneC = _noneC()

def _boolC() -> Codec:
    intC_encode, intC_decode = intC

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

def maybe(codec : Codec) -> Codec:
    enc, dec = codec
    boolC_encode, boolC_decode = boolC

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

def newtypeC(codec : Codec, ctor, proj) -> Codec:
    enc, dec = codec

    def encode(f : FileOut, x : Any) -> None:
        enc(f, proj(x))

    def decode(f : FileIn) -> Any:
        return ctor(dec(f))

    return Codec(encode, decode)

def numpyC(dtype : type) -> Codec:
    bytesC_enc, bytesC_dec = bytesC
    l_enc, l_dec = listC(intC)

    def encode(f : FileOut, x : np.array) -> None:
        assert x.dtype == dtype, f"expected array type: {dtype}, received: {x.dtype}"
        l_enc(f, x.shape)
        bytesC_enc(f, x.tobytes())

    def decode(f : FileIn) -> np.array:
        shape = tuple(l_dec(f))
        stuff = bytesC_dec(f)
        return np.reshape(
            np.fromstring(stuff, dtype=dtype),
            newshape=shape,
        )
    
    return Codec(encode, decode)
