from dataclasses import dataclass
from typing import Callable, Sequence, TypeVar, Generic, Any
from gui.progress import Worker, MockWorker, Cancelled
from util.codec import Codec, FileIn, FileOut, intC, strC, CodecError

T = TypeVar('T')

@dataclass
class CodecProgress(Generic[T]):
    # number of times the codec will call worker.step()
    get_size : Callable[[T], int]

    encode : Callable[[Worker, FileOut, T], None]
    decode : Callable[[Worker, FileIn], T]

    def enc_dec(self) -> tuple[
        Callable[[T], int],
        Callable[[Worker, FileOut, T], None],
        Callable[[Worker, FileIn], T],
    ]: 
        return self.get_size, self.encode, self.decode

def oneCP(codec : Codec[T]) -> CodecProgress[T]:
    enc, dec = codec.enc_dec()

    def get_size(x : T) -> int:
        return 1

    def encode(worker : Worker, f : FileOut, x : T) -> None:
        enc(f, x)
        worker.step()

    def decode(worker : Worker, f : FileIn) -> T:
        result = dec(f)
        worker.step()
        return result

    return CodecProgress(get_size, encode, decode)

def listCP(codec : CodecProgress[T]) -> CodecProgress[list[T]]:
    get_sz, enc, dec = codec.enc_dec()
    intC_enc, intC_dec = intC.enc_dec()

    def get_size(xs : list[T]) -> int:
        return sum(get_sz(x) for x in xs)

    def encode(worker : Worker, f : FileOut, xs : list[T]) -> None:
        intC_enc(f, len(xs))
        for x in xs:
            enc(worker, f, x)  # calls worker.step()

    def decode(worker : Worker, f : FileIn) -> list[T]:
        length = intC_dec(f)
        result = []
        for _ in range(length):
            result.append(dec(worker, f))
        return result

    return CodecProgress(get_size, encode, decode)

def enum_by_typenameCP(name : str, alts : Sequence[tuple[type, CodecProgress]]) -> CodecProgress:
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

    strC_encode, strC_decode = strC.enc_dec()

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
