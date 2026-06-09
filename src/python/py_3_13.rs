use pyo3::ffi::{Py_ASNATIVEBYTES_UNSIGNED_BUFFER, PyLong_FromUnsignedNativeBytes, PyObject};

use crate::parse::uuid_to_bytes;

#[cfg(target_endian = "big")]
const ENDIAN_FLAG: i32 = pyo3::ffi::Py_ASNATIVEBYTES_BIG_ENDIAN;

#[cfg(target_endian = "little")]
const ENDIAN_FLAG: i32 = pyo3::ffi::Py_ASNATIVEBYTES_LITTLE_ENDIAN;

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut buf = [0_u8; 16];

    unsafe {
        uuid_to_bytes(hi, lo, buf.as_mut_ptr());
        PyLong_FromUnsignedNativeBytes(
            buf.as_ptr().cast(),
            buf.len(),
            ENDIAN_FLAG | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        )
    }
}
