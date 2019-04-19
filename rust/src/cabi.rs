use std::os::raw::{c_int, c_char, c_double};
use std::ffi::{CStr, CString};
use crate::{Store, Result, ErrorKind};
use crate::bridge::*;

#[no_mangle]
pub unsafe extern "C" fn yahtzeevalue_init() {
    set_panic_hook();
}

export!(yahtzeevalue_free(buf: *mut c_char) -> Result<c_int> {
    CString::from_raw(buf);
    Ok(0)
});

export!(yahtzeevalue_load(path: *const c_char) -> Result<*mut Store> {
    Ok(Box::into_raw(Box::new(Store::new(CStr::from_ptr(path).to_str()?)?)))
});

export!(yahtzeevalue_unload(db: *mut Store) -> Result<c_int> {
    Box::from_raw(db);
    Ok(0)
});

export!(yahtzeevalue_lookup(db: *mut Store, key: c_int) -> Result<c_double> {
    let key = key as u32;
    if key >= (*db).len() {
        return Err(ErrorKind::Range.into());
    }
    Ok((*db).get(key))
});
