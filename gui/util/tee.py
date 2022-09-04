from typing import Any, cast

class Tee:
    # no clue how to type this correctly
    def __init__(self, f : Any, f_tee : Any) -> None:
        self.f = f
        self.f_tee = f_tee

    def read(self, nbytes : int) -> bytes:
        buf = self.f.read(nbytes)
        self.f_tee.write(buf)
        return cast(bytes, buf)

    def write(self, buf : bytes) -> None:
        self.f_tee.write(buf)
        self.f.write(buf)

    def flush(self) -> None:
        self.f.flush()
