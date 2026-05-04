use std::ffi::c_uchar;

use pyo3::ffi::{_PyLong_FromByteArray, PyObject};

use crate::parse::uuid_to_bytes;

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut bytes = [0u8; 16];
    unsafe {
        uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
        _PyLong_FromByteArray(bytes.as_ptr().cast::<c_uchar>(), 16, 0, 0)
    }
}
