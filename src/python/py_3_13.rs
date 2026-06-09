use pyo3::ffi::{
    Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER, PyLong_FromUnsignedNativeBytes,
    PyObject,
};

use crate::parse::uuid_to_bytes;

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut buf = [0_u8; size_of::<u128>()];

    unsafe {
        uuid_to_bytes(hi, lo, buf.as_mut_ptr());
        PyLong_FromUnsignedNativeBytes(
            buf.as_ptr().cast(),
            buf.len(),
            Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        )
    }
}
