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


_special_errors = {
    1: UnicodeDecodeError,
    2: RangeError,
    3: IOError,
    4: FileNotFoundError,
}

_rustcall = _bridge.make_rustcall(
    "struct yahtzeevalue_error *", _lib.yahtzeevalue_free, _special_errors, _ffi
)


class Database:
    def __init__(self, path):
        self._path = path

    def __enter__(self):
        self._handle = _rustcall(_lib.yahtzeevalue_load, self._path.encode('utf-8'))
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        _rustcall(_lib.yahtzeevalue_unload, self._handle)
        del self._handle

    def __getitem__(self, key):
        return _rustcall(_lib.yahtzeevalue_lookup, self._handle, key)
