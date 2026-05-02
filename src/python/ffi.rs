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
mod py_3_14 {
    use std::{
        ffi::{c_int, c_void},
        ptr::{addr_of_mut, null_mut},
    };

    use pyo3::ffi::{Py_ssize_t, PyObject};

    // https://docs.python.org/3/c-api/long.html#c.PyLongWriter
    #[repr(C)]
    struct PyLongWriter {
        _opaque: [u8; 0],
    }

    unsafe extern "C" {
        // https://docs.python.org/3/c-api/long.html#c.PyLongWriter_Create
        fn PyLongWriter_Create(
            negative: c_int,
            ndigits: Py_ssize_t,
            digits: *mut *mut c_void,
        ) -> *mut PyLongWriter;

        fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;
    }

    pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
        const SHIFT: u32 = 30;
        const MASK: u64 = (1 << SHIFT) - 1;

        let mut ptr: *mut c_void = null_mut();

        let writer = unsafe { PyLongWriter_Create(0, 5, addr_of_mut!(ptr)) };

        if writer.is_null() {
            return null_mut();
        }

        let digit = ptr.cast::<u32>();
        unsafe {
            digit.write((lo & MASK) as u32);
            digit.add(1).write(((lo >> 30) & MASK) as u32);
            digit.add(2).write((((lo >> 60) | (hi << 4)) & MASK) as u32);
            digit.add(3).write(((hi >> 26) & MASK) as u32);
            digit.add(4).write((hi >> 56) as u32);
            PyLongWriter_Finish(writer)
        }
    }
}

#[cfg(all(Py_3_13, not(all(Py_3_14, not(PyPy)))))]
mod py_3_13 {
    use std::ffi::c_void;

    use pyo3::ffi::{
        Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        PyLong_FromUnsignedNativeBytes, PyObject,
    };

    use crate::parse::uuid_to_bytes;

    pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
        let mut bytes = [0u8; 16];
        unsafe {
            uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
            PyLong_FromUnsignedNativeBytes(
                bytes.as_ptr().cast::<c_void>(),
                16,
                Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
            )
        }
    }
}

#[cfg(not(Py_3_13))]
mod pre_py_3_13 {
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
}

#[cfg(not(Py_3_13))]
pub use pre_py_3_13::uuid_int_from_parts;
//
#[cfg(all(Py_3_13, not(all(Py_3_14, not(PyPy)))))]
pub use py_3_13::uuid_int_from_parts;
//
#[cfg(all(Py_3_14, not(PyPy)))]
pub use py_3_14::uuid_int_from_parts;
