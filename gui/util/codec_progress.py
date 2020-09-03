from typing import Callable, Any, NamedTuple, List, Sequence, Tuple

from gui.progress import Worker, MockWorker, Cancelled
from util.codec import Codec, FileIn, FileOut, intC, strC, CodecError

class CodecProgress(NamedTuple):
    # number of times the codec will call worker.step()
    get_size : Callable[[Any], int]

    encode : Callable[[Worker, FileOut, Any], None]
    decode : Callable[[Worker, FileIn], Any]

def oneCP(codec : Codec) -> CodecProgress:
    enc, dec = codec

    def get_size(x : Any) -> int:
        return 1

    def encode(worker : Worker, f : FileOut, x : Any) -> None:
        enc(f, x)
        worker.step()

    def decode(worker : Worker, f : FileIn) -> Any:
        result = dec(f)
        worker.step()
        return result

    return CodecProgress(get_size, encode, decode)

def listCP(codec : CodecProgress):
    get_sz, enc, dec = codec
    intC_enc, intC_dec = intC

    def get_size(xs : List[Any]) -> int:
        return sum(get_sz(x) for x in xs)

    def encode(worker : Worker, f : FileOut, xs : List[Any]) -> None:
        intC_enc(f, len(xs))
        for x in xs:
            enc(worker, f, x)  # calls worker.step()

    def decode(worker : Worker, f : FileIn) -> List[Any]:
        length = intC_dec(f)
        result = []
        for _ in range(length):
            result.append(dec(worker, f))
        return result

    return CodecProgress(get_size, encode, decode)

def enum_by_typenameCP(name : str, alts : Sequence[Tuple[type, CodecProgress]]) -> CodecProgress:
    codecs_sz_get = {
        ty.__name__: codec.get_size
        for ty, codec in alts
    }.get

    codecs_enc_get = {
        ty.__name__: codec.encode
        for ty, codec in alts
    }.get

    codecs_dec_get = {
        ty.__name__: codec.decode
        for ty, codec in alts
    }.get

    strC_encode, strC_decode = strC

    def get_size(x : tuple) -> int:
        get_sz = codecs_sz_get(type(x).__name__)
        assert get_sz
        return get_sz(x)

    def encode(worker : Worker, f : FileOut, x : tuple) -> None:
        ty = type(x).__name__
        enc = codecs_enc_get(ty)
        if enc is None:
            raise CodecError(f'cannot encode enum class: {name}/{ty}')

        strC_encode(f, ty)
        enc(worker, f, x)

    def decode(worker : Worker, f : FileIn) -> Any:
        ty = strC_decode(f)
        dec = codecs_dec_get(ty)
        if dec is None:
            raise CodecError(f'cannot decode enum class: {name}/{ty}')

        return dec(worker, f)

    return CodecProgress(get_size, encode, decode)
