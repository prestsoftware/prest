from typing import Iterable, TypeVar, Generic, Iterator, Callable, overload, Union
from util.codec import Codec, FileIn, FileOut, listC, bytesC

T = TypeVar('T')

class PackedList(Generic[T]):
    def __init__(self, codec : Codec, elms : Iterable[T] = ()) -> None:
        self.enc : Callable[[T], bytes] = codec.encode_to_memory
        self.dec : Callable[[bytes], T] = codec.decode_from_memory
        self.blocks : list[bytes] = []

        self.extend(elms)

    def extend(self, xs : Iterable[T]) -> None:
        if isinstance(xs, PackedList):
            self.blocks.extend(xs.blocks)
        else:
            self.blocks.extend(map(self.enc, xs))

    def append(self, x : T) -> None:
        self.blocks.append(self.enc(x))

    def append_packed(self, x : bytes) -> None:
        self.blocks.append(x)

    def get_packed(self, idx : int) -> bytes:
        return self.blocks[idx]

    def __iter__(self) -> Iterator[T]:
        return iter(map(self.dec, self.blocks))

    def __len__(self) -> int:
        return len(self.blocks)

    @overload
    def __getitem__(self, idx : int) -> T:
        pass

    @overload
    def __getitem__(self, idx : slice) -> Iterable[T]:
        pass

    def __getitem__(self, idx : Union[int, slice]) -> Union[T, Iterable[T]]:
        if isinstance(idx, int):
            return self.dec(self.blocks[idx])
        elif isinstance(idx, slice):
            return map(self.dec, self.blocks[idx])
        else:
            raise ValueError('bad index: %s' % idx)

def PackedListC(codec : Codec) -> Codec:
    enc, dec = listC(bytesC).enc_dec()

    def encode(f : FileOut, xs : PackedList) -> None:
        enc(f, xs.blocks)

    def decode(f : FileIn) -> PackedList:
        xs : PackedList = PackedList(codec)
        xs.blocks = dec(f)
        return xs

    return Codec(encode, decode)
