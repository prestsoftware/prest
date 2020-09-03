class AbstractWorker:
    def __init__(self):
        raise NotImplementedError()

    def set_work_size(self, _size : int) -> None:
        raise NotImplementedError()

    def set_progress(self, _value : int) -> None:
        raise NotImplementedError()

class MockWorker(AbstractWorker):
    def __init__(self):
        pass

    def set_work_size(self, _size : int) -> None:
        pass

    def set_progress(self, _value : int) -> None:
        pass
