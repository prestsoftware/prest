class Tee:
    def __init__(self, f, f_tee):
        self.f = f
        self.f_tee = f_tee

    def read(self, nbytes):
        buf = self.f.read(nbytes)
        self.f_tee.write(buf)
        return buf

    def write(self, buf):
        self.f_tee.write(buf)
        self.f.write(buf)

    def flush(self):
        self.f.flush()
