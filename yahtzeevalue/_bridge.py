# Generalization of https://youtu.be/zmtHaZG7pPc?t=22m29s
def make_rustcall(error_type, error_message_free, special_errors, ffi):
    """
    Helper function for calling Rust functions.

    - error_type: string naming a struct with 'failed', 'code', 'message'
    - error_message_free: cffi function to free a 'const char *'.
    - special_errors: dict mapping code (int) to Exception type
    - ffi: cffi glue module with 'new' and 'string' functions

    struct error_type_example {
        int failed;
        int code;
        const char * message;
    };

    void error_message_free_example(const char * p) {
        free(p);
    }
    """

    def rustcall(func, *args):
        err = ffi.new(error_type)
        rv = func(*(args + (err,)))
        if not err[0].failed:
            return rv
        try:
            exc_class = special_errors.get(err[0].code, Exception)
            exc = exc_class(ffi.string(err[0].message).decode('utf-8', 'replace'))
        finally:
            error_message_free(err[0].message)
        raise exc

    return rustcall
