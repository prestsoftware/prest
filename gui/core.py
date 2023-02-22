import io
import os
import sys
import select
import logging
import threading
import subprocess
import collections
import typing
from typing import Sequence, Any, Type, Optional, NamedTuple, Union, BinaryIO, cast, TypeVar

import model
import platform_specific
from util.tee import Tee

from util.codec import Codec, CodecError, EOF, FileIn, FileOut, namedtupleC, \
    strC, intC, frozensetC, listC, bytesC, tupleC, enumC

log = logging.getLogger(__name__)

Req = TypeVar('Req')
Resp = TypeVar('Resp')

class CoreError(Exception):
    pass

class CoreDeath(CoreError):
    pass

class MalformedResponse(CoreError):
    pass

class Progress(NamedTuple):
    position : int
    tag : int = 0

class AnswerFollows(NamedTuple):
    tag : int = 1

class Error(NamedTuple):
    message : str
    extra : bytes
    tag : int = 2

class Log(NamedTuple):
    level : int
    message : str
    tag : int = 3

Message = Union[
    Progress,
    AnswerFollows,
    Error,
    Log,
]

MessageC = enumC('Message', {
    Progress: (intC,),
    AnswerFollows: (),
    Error: (strC, bytesC),
    Log: (intC, strC),
})

class Failure(CoreError):
    def __init__(self, message : str, error : bytes) -> None:
        CoreError.__init__(self, message)
        self.error = error

class StreamReader(threading.Thread):
    def __init__(self, f : FileIn) -> None:
        threading.Thread.__init__(self, daemon=True)
        self.f = f
        self.buf = io.BytesIO()

    def run(self) -> None:
        while True:
            xs = self.f.read()
            if not xs:
                break
            self.buf.write(xs)

    def get_content(self) -> bytes:
        return self.buf.getvalue()

class Core:
    def __init__(
        self,
        f_tee: Optional[tuple] = None,
        f_mock: Optional[Any] = None,
    ) -> None:
        log.debug('creating core')

        fname_precomputed_preorders : Optional[str]
        try:
            fname_precomputed_preorders = platform_specific.get_embedded_file_path('preorders-7.bin')
        except platform_specific.FileNotFound:
            fname_precomputed_preorders = None

        cmdline = [platform_specific.get_embedded_file_path(
            'prest-core.exe',  # deployment Windows
            'prest-core',      # deployment elsewhere (?)
            'prest-core/target/release/prest-core.exe',  # CI Windows
            'prest-core/target/release/prest-core',      # CI elsewhere
        )]
        if fname_precomputed_preorders:
            cmdline += ['--precomputed-preorders', fname_precomputed_preorders]

        try:
            self.core = subprocess.Popen(
                cmdline,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            self.stdin : FileOut
            self.stdout : FileIn

            if f_tee:
                f_in, f_out = f_tee
                assert self.core.stdin
                assert self.core.stdout
                self.stdin = cast(FileOut, Tee(self.core.stdin, f_in))
                self.stdout = cast(FileIn, Tee(self.core.stdout, f_out))
            else:
                self.stdin = typing.cast(FileOut, self.core.stdin)

                if f_mock:
                    self.stdout = typing.cast(FileIn, f_mock)
                else:
                    self.stdout = typing.cast(FileIn, self.core.stdout)

            self.stderr : FileIn = typing.cast(FileIn, self.core.stderr)
        except OSError:
            raise CoreError('could not run core')

        self.stderr_reader = StreamReader(self.stderr)
        self.stderr_reader.start()

        log.debug('the core is running')

    def __enter__(self) -> 'Core':
        return self

    def __exit__(self, *_exc_info : Any) -> None:
        self.shutdown()

    def call(self, name : str, codec_req : Codec[Req], codec_resp : Codec[Resp], request : Req) -> Resp:
        strC.encode(self.stdin, name)
        codec_req.encode(self.stdin, request)
        self.stdin.flush()

        try:
            while True:
                msg = MessageC.decode(self.stdout)
                #log.debug('message received: %s' % str(msg))

                if isinstance(msg, Progress):
                    log.debug('progress: %d' % msg.position)

                elif isinstance(msg, Log):
                    level = ['DEBUG', 'INFO', 'WARN', 'ERROR'][msg.level]
                    log.info('[%s] %s' % (level, msg.message))

                elif isinstance(msg, AnswerFollows):
                    return codec_resp.decode(self.stdout)

                elif isinstance(msg, Error):
                    raise Failure(msg.message, msg.extra)

                else:
                    raise MalformedResponse('invalid response: %s' % msg)
        except EOF:
            death_note = self.stderr_reader.get_content().decode('utf8')
            log.warn('core died with message: {0}'.format(death_note))
            raise CoreDeath(death_note)
        except CodecError as e:
            stderr = self.stderr_reader.get_content().decode('utf8')
            log.warn('core stderr: {0}'.format(stderr))
            raise MalformedResponse('malformed response from core') from e
        except Exception:
            stderr = self.stderr_reader.get_content().decode('utf8')
            log.warn('core stderr: {0}'.format(stderr))
            raise

    def crash(self) -> None:
        self.call('crash', strC, strC, 'Crash test')

    def soft_failure(self) -> None:
        self.call('fail', strC, strC, 'Failure test')

    def shutdown(self) -> None:
        log.debug('core shutdown')

        try:
            strC.encode(self.stdin, 'quit')
            self.stdin.flush()
        except (OSError, BrokenPipeError):  # windows throws OSError
            log.debug('could not send quit, the core is probably dead already')

        try:
            self.core.wait(2)  # seconds
        except subprocess.TimeoutExpired:
            log.warn("core won't quit, killing")
            self.core.terminate()

            try:
                self.core.wait(2)  # seconds
            except subprocess.TimeoutExpired:
                log.error("core does not respond to SIGTERM, sending SIGKILL and not waiting anymore")
                self.core.kill()

        log.debug('waiting for stderr reader...')
        self.stderr_reader.join(1)  # join the stderr reader

        if self.stderr_reader.is_alive():
            log.warn("stderr reader won't quit, leaking it")
        else:
            log.debug('stderr reader joined')
