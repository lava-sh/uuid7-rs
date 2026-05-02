#[rustfmt::skip]
use pyo3::ffi::PyObject;
//
#[cfg(not(Py_3_13))]
pub use pyo3::ffi::_PyLong_AsByteArray as PyLong_AsNativeBytes;
//
#[cfg(Py_3_13)]
pub use pyo3::ffi::PyLong_AsNativeBytes;
//
#[cfg(not(PyPy))]
pub use pyo3::ffi::{PyTuple_GET_ITEM, PyTuple_GET_SIZE, PyTuple_SET_ITEM};
//
#[cfg(PyPy)]
pub use pyo3::ffi::{
    PyTuple_GetItem as PyTuple_GET_ITEM, PyTuple_SetItem as PyTuple_SET_ITEM,
    PyTuple_Size as PyTuple_GET_SIZE,
};

#[cfg(all(Py_3_14, not(PyPy)))]
#[repr(C)]
struct PyLongWriter {
    _opaque: [u8; 0],
}

#[cfg(all(Py_3_14, not(PyPy)))]
unsafe extern "C" {
    fn PyLongWriter_Create(
        negative: std::ffi::c_int,
        ndigits: pyo3::ffi::Py_ssize_t,
        digits: *mut *mut std::ffi::c_void,
    ) -> *mut PyLongWriter;

    fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;
}

#[cfg(all(Py_3_14, not(PyPy)))]
pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    use std::{
        ffi::c_void,
        ptr::{addr_of_mut, null_mut},
    };

    const SHIFT: u32 = 30;
    const MASK: u64 = (1 << SHIFT) - 1;

    let mut digits_ptr: *mut c_void = null_mut();
    let writer = unsafe { PyLongWriter_Create(0, 5, addr_of_mut!(digits_ptr)) };
    if writer.is_null() {
        return null_mut();
    }

    let d = digits_ptr.cast::<u32>();
    unsafe {
        d.write((lo & MASK) as u32); // bits 0..29
        d.add(1).write(((lo >> 30) & MASK) as u32); // bits 30..59
        d.add(2).write((((lo >> 60) | (hi << 4)) & MASK) as u32); // bits 60..89
        d.add(3).write(((hi >> 26) & MASK) as u32); // bits 90..119
        d.add(4).write((hi >> 56) as u32); // bits 120..127
        PyLongWriter_Finish(writer)
    }
}

#[cfg(all(Py_3_13, not(all(Py_3_14, not(PyPy)))))]
pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    use std::ffi::c_void;

    use pyo3::ffi::{
        Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        PyLong_FromUnsignedNativeBytes,
    };

    let mut bytes = [0u8; 16];
    unsafe {
        crate::parse::uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
        PyLong_FromUnsignedNativeBytes(
            bytes.as_ptr().cast::<c_void>(),
            16,
            Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        )
    }
}

#[cfg(not(Py_3_13))]
pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    use std::ffi::c_uchar;

    use pyo3::ffi::_PyLong_FromByteArray;

    let mut bytes = [0u8; 16];
    unsafe {
        crate::parse::uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
        _PyLong_FromByteArray(bytes.as_ptr().cast::<c_uchar>(), 16, 0, 0)
    }
}
