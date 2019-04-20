use std::os::raw::{c_int, c_char, c_double};
use std::ffi::{CStr, CString};
use crate::{Store, Result, ErrorKind, Outcome};
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

export!(yahtzeevalue_lookup(db: *mut Store, state: c_int) -> Result<c_double> {
    let state = state as u32;
    if state >= (*db).len() {
        return Err(ErrorKind::Range.into());
    }
    Ok((*db).get(state))
});

export!(yahtzeevalue_best_action(db: *mut Store, state: c_int, histogram: c_int) -> Result<c_int> {
    let state = state as u32;
    if state >= (*db).len() {
        return Err(ErrorKind::Range.into());
    }
    let outcome = Outcome::decode(histogram as u32);
    (*db).best_action(state, outcome).map(|v| v as c_int).ok_or(ErrorKind::GameOver.into())
});

export!(yahtzeevalue_keep_first(db: *mut Store, state: c_int, histogram: c_int) -> Result<c_int> {
    let state = state as u32;
    if state >= (*db).len() {
        return Err(ErrorKind::Range.into());
    }
    let outcome = Outcome::decode(histogram as u32);
    Ok((*db).keep_first(state, outcome) as c_int)
});

export!(yahtzeevalue_keep_second(db: *mut Store, state: c_int, histogram: c_int) -> Result<c_int> {
    let state = state as u32;
    if state >= (*db).len() {
        return Err(ErrorKind::Range.into());
    }
    let outcome = Outcome::decode(histogram as u32);
    Ok((*db).keep_second(state, outcome) as c_int)
});
