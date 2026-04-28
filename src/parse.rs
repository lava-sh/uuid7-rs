use std::{
    os::raw::{c_char, c_int},
    ptr::{addr_of_mut, copy_nonoverlapping, null_mut, read_unaligned, write_unaligned},
    slice::from_raw_parts,
};

use pyo3::ffi::{
    Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER, Py_DECREF, Py_None, Py_ssize_t,
    PyBytes_AsStringAndSize, PyErr_Clear, PyErr_ExceptionMatches, PyErr_Format, PyErr_Occurred,
    PyExc_OverflowError, PyExc_TypeError, PyList_Check, PyList_GET_ITEM, PyLong_AsUnsignedLongLong,
    PyLong_Check, PyObject, PySequence_Fast, PySequence_Size, PyUnicode_AsUTF8AndSize,
    PyUnicode_Check,
};

use crate::{
    hex::helpers::parse_uuid_hex_str,
    python::{
        exceptions::{PyTypeError, PyValueError},
        ffi::{PyLong_AsNativeBytes, PyTuple_GET_ITEM},
    },
};

#[inline]
pub fn bytes_to_hilo(bytes: *const u8, hi: &mut u64, lo: &mut u64) {
    *hi = u64::from_be(unsafe { read_unaligned(bytes.cast::<u64>()) });
    *lo = u64::from_be(unsafe { read_unaligned(bytes.add(8).cast::<u64>()) });
}

#[inline]
pub fn uuid_to_bytes(hi: u64, lo: u64, out: *mut u8) {
    unsafe {
        write_unaligned(out.cast::<u64>(), hi.to_be());
        write_unaligned(out.add(8).cast::<u64>(), lo.to_be());
    }
}

pub fn uuid_to_bytes_le(bytes: &[u8; 16], out: &mut [u8; 16]) {
    out[0] = bytes[3];
    out[1] = bytes[2];
    out[2] = bytes[1];
    out[3] = bytes[0];
    out[4] = bytes[5];
    out[5] = bytes[4];
    out[6] = bytes[7];
    out[7] = bytes[6];
    out[8..].copy_from_slice(&bytes[8..]);
}

#[inline]
pub fn uuid_to_bytes_le_ptr(src: *const u8, dst: *mut u8) {
    unsafe {
        *dst.add(0) = *src.add(3);
        *dst.add(1) = *src.add(2);
        *dst.add(2) = *src.add(1);
        *dst.add(3) = *src.add(0);
        *dst.add(4) = *src.add(5);
        *dst.add(5) = *src.add(4);
        *dst.add(6) = *src.add(7);
        *dst.add(7) = *src.add(6);
        copy_nonoverlapping(src.add(8), dst.add(8), 8);
    }
}

pub fn parse_uuid(value: *mut PyObject, hi: &mut u64, lo: &mut u64) -> c_int {
    if unsafe { PyUnicode_Check(value) } == 0 {
        PyTypeError::new_err(c"UUID() argument must be a str");
        return -1;
    }
    let mut size: Py_ssize_t = 0;
    let text = unsafe { PyUnicode_AsUTF8AndSize(value, addr_of_mut!(size)) };

    if text.is_null() {
        return -1;
    }

    let slice = unsafe { from_raw_parts(text.cast::<u8>(), size.cast_unsigned()) };
    if parse_uuid_hex_str(slice, hi, lo) != 0 {
        PyValueError::new_err(c"badly formed hexadecimal UUID string");
        return -1;
    }
    0
}

pub fn parse_uuid_bytes(value: *mut PyObject, le: bool, hi: &mut u64, lo: &mut u64) -> c_int {
    let mut buf: *mut c_char = null_mut();
    let mut len: Py_ssize_t = 0;
    if unsafe { PyBytes_AsStringAndSize(value, addr_of_mut!(buf), addr_of_mut!(len)) } != 0 {
        PyTypeError::new_err(c"bytes must be a 16-char bytes object");
        return -1;
    }
    if len != 16 {
        PyValueError::new_err(c"bytes is not a 16-char string");
        return -1;
    }
    let p = buf as *const u8;
    if le {
        let mut reordered = [0u8; 16];
        uuid_to_bytes_le_ptr(p, reordered.as_mut_ptr());
        bytes_to_hilo(reordered.as_ptr(), hi, lo);
    } else {
        bytes_to_hilo(p, hi, lo);
    }
    0
}

pub fn parse_uuid_int(value: *mut PyObject, hi: &mut u64, lo: &mut u64) -> c_int {
    if unsafe { PyLong_Check(value) } == 0 {
        PyTypeError::new_err(c"int must be a 128-bit integer");
        return -1;
    }
    let mut bytes = [0u8; 16];
    #[cfg(Py_3_13)]
    let rc = unsafe {
        PyLong_AsNativeBytes(
            value,
            bytes.as_mut_ptr().cast::<std::os::raw::c_void>(),
            16,
            Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
        )
    };
    #[cfg(not(Py_3_13))]
    let rc = unsafe {
        PyLong_AsNativeBytes(
            value.cast::<pyo3::ffi::PyLongObject>(),
            bytes.as_mut_ptr().cast::<std::os::raw::c_uchar>(),
            16,
            0,
            0,
        )
    };
    if rc < 0 {
        if unsafe { PyErr_ExceptionMatches(PyExc_OverflowError) } != 0 {
            PyValueError::new_err(c"int is out of range (need a 128-bit value)");
        }
        return -1;
    }
    #[cfg(Py_3_13)]
    if rc > 16 {
        PyValueError::new_err(c"int is out of range (need a 128-bit value)");
        return -1;
    }
    bytes_to_hilo(bytes.as_ptr(), hi, lo);
    0
}

pub fn parse_uuid_fields(value: *mut PyObject, hi: &mut u64, lo: &mut u64) -> c_int {
    static LIMITS: [u64; 6] = [0xFFFF_FFFF, 0xFFFF, 0xFFFF, 0xFF, 0xFF, 0xFFFF_FFFF_FFFF];

    let fast = unsafe { PySequence_Fast(value, c"fields must be a 6-tuple".as_ptr()) };
    if fast.is_null() {
        return -1;
    }
    let size = unsafe { PySequence_Size(fast) };
    if size != 6 {
        unsafe { Py_DECREF(fast) };
        PyValueError::new_err(c"fields is not a 6-tuple");
        return -1;
    }
    let mut parts = [0u64; 6];

    for i in 0usize..6 {
        #[cfg(not(PyPy))]
        let item = if unsafe { PyList_Check(fast) } == 1 {
            unsafe { PyList_GET_ITEM(fast, i.cast_signed()) }
        } else {
            unsafe { PyTuple_GET_ITEM(fast, i.cast_signed()) }
        };
        #[cfg(PyPy)]
        let item = unsafe { PySequence_GetItem(fast, i.cast_signed()) };
        #[cfg(PyPy)]
        if item.is_null() {
            unsafe { Py_DECREF(fast) };
            return -1;
        }
        let v = unsafe { PyLong_AsUnsignedLongLong(item) };
        unsafe {
            #[cfg(PyPy)]
            Py_DECREF(item);
            if !PyErr_Occurred().is_null() {
                Py_DECREF(fast);
                PyErr_Clear();
                PyTypeError::new_err(c"fields must contain only integers");
                return -1;
            }
        }

        if v > LIMITS[i] {
            unsafe { Py_DECREF(fast) };
            PyValueError::new_err(c"field value out of range");
            return -1;
        }
        parts[i] = v;
    }
    unsafe { Py_DECREF(fast) };
    *hi = (parts[0] << 32) | (parts[1] << 16) | parts[2];
    *lo = (parts[3] << 56) | (parts[4] << 48) | parts[5];
    0
}

#[inline]
pub fn parse_u64_arg(value: *mut PyObject, name: *const c_char) -> (c_int, u64) {
    if value.is_null() || value == unsafe { Py_None() } {
        return (0, 0);
    }

    let v = unsafe { PyLong_AsUnsignedLongLong(value) };
    if !unsafe { PyErr_Occurred() }.is_null() {
        unsafe {
            PyErr_Clear();
            PyErr_Format(
                PyExc_TypeError,
                c"%s must be a non-negative int or None".as_ptr(),
                name,
            );
        }
        return (-1, 0);
    }
    (1, v)
}
