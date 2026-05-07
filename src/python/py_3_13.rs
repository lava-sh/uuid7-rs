use std::ffi::c_void;

use pyo3::ffi::{
    Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER, PyLong_FromUnsignedNativeBytes,
    PyObject,
};

use crate::parse::uuid_to_bytes;

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut bytes = [0_u8; 16];
    unsafe {
        uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
        PyLong_FromUnsignedNativeBytes(
            bytes.as_ptr().cast::<c_void>(),
            16,
            Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        )
    }
}
