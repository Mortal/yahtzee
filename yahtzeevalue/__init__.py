from ._native import lib as _lib, ffi as _ffi
from . import _bridge


__all__ = ["YahtzeeError", "YahtzeeValue"]

_lib.yahtzeevalue_init()


class FstrieError(Exception):
    pass


class UnicodeDecodeError(FstrieError):
    pass


class RootDoesNotExistError(FstrieError):
    pass


class IOError(FstrieError):
    pass


_special_errors = {
    1: UnicodeDecodeError,
    2: RootDoesNotExistError,
    3: IOError,
}

_rustcall = _bridge.make_rustcall(
    "struct fstrie_error *", _lib.fstrie_free, _special_errors, _ffi
)


class Database:
    def __init__(self, root):
        self._root = root

    def __enter__(self):
        self._handle = _rustcall(_lib.fstrie_load, self._root.encode('utf-8'))
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        _rustcall(_lib.fstrie_unload, self._handle)
        del self._handle

    def __getitem__(self, key):
        result_ptr = _rustcall(_lib.fstrie_lookup, self._handle, key.encode('utf-8'))
        try:
            result = []
            i = 0
            while result_ptr[i] != _ffi.NULL:
                result.append(_ffi.string(result_ptr[i]).decode('utf-8', 'replace'))
                i += 1
            return result
        finally:
            _lib.fstrie_free_list(result_ptr)

