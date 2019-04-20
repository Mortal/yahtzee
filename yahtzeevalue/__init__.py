from ._native import lib as _lib, ffi as _ffi
from . import _bridge


__all__ = ["YahtzeeError", "YahtzeeValue"]

_lib.yahtzeevalue_init()


class YahtzeeError(Exception):
    pass


class UnicodeDecodeError(YahtzeeError):
    pass


class RangeError(YahtzeeError):
    pass


class IOError(YahtzeeError):
    pass


class FileNotFoundError(YahtzeeError):
    pass


class GameOverError(YahtzeeError):
    pass


_special_errors = {
    1: UnicodeDecodeError,
    2: RangeError,
    3: IOError,
    4: FileNotFoundError,
    5: GameOverError,
}

_rustcall = _bridge.make_rustcall(
    "struct yahtzeevalue_error *", _lib.yahtzeevalue_free, _special_errors, _ffi
)


def encode_roll(roll):
    histogram = [0] * 6
    for v in roll:
        histogram[v - 1] += 1
    encoding = 0
    acc = 1
    for i, v in enumerate(histogram):
        encoding += acc * v
        acc *= 7
    return encoding


def decode_roll(encoding):
    histogram = [0] * 6
    for i in range(len(histogram)):
        encoding, res = divmod(encoding, 7)
        histogram[i] = res
    return [i + 1 for i, v in enumerate(histogram) for _ in range(v)]


class Database:
    def __init__(self, path):
        self._path = path

    def __enter__(self):
        self._handle = _rustcall(_lib.yahtzeevalue_load, self._path.encode('utf-8'))
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        _rustcall(_lib.yahtzeevalue_unload, self._handle)
        del self._handle

    def lookup(self, state):
        return _rustcall(_lib.yahtzeevalue_lookup, self._handle, state)

    def best_action(self, state, histogram):
        return _rustcall(_lib.yahtzeevalue_best_action, self._handle, state, encode_roll(histogram))

    def keep_first(self, state, histogram):
        return decode_roll(_rustcall(_lib.yahtzeevalue_keep_first, self._handle, state, encode_roll(histogram)))

    def keep_second(self, state, histogram):
        return decode_roll(_rustcall(_lib.yahtzeevalue_keep_second, self._handle, state, encode_roll(histogram)))
