use pyo3::ffi::{_PyLong_FromByteArray, PyObject};

use crate::parse::uuid_to_bytes;

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut buf = [0_u8; size_of::<u128>()];

    unsafe {
        uuid_to_bytes(hi, lo, buf.as_mut_ptr());
        _PyLong_FromByteArray(buf.as_ptr().cast(), buf.len(), 0, 0)
    }
}
