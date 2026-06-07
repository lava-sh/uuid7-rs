use std::{
    ffi::{CStr, c_char, c_int},
    ptr::{addr_of_mut, copy_nonoverlapping, null_mut, read_unaligned, write_unaligned},
    slice::from_raw_parts,
};

use pyo3::ffi::{
    Py_DECREF, Py_None, Py_ssize_t, PyBytes_AsStringAndSize, PyErr_Clear, PyErr_Format,
    PyErr_Occurred, PyExc_TypeError, PyLong_AsUnsignedLongLong, PyLong_Check, PyObject,
    PyObject_Length, PySequence_Fast, PyUnicode_AsUTF8AndSize, PyUnicode_Check,
};

use crate::{
    hex::helpers::parse_uuid_hex_str,
    python::{
        exceptions::{PyTypeError, PyValueError},
        ffi::PySequence_Fast_GET_SIZE,
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
    let len = unsafe { PyObject_Length(value) };

    if len < 0 {
        return -1;
    }

    if len != 16 {
        let msg = if le {
            c"bytes_le is not a 16-char string"
        } else {
            c"bytes is not a 16-char string"
        };
        PyValueError::new_err(msg);
        return -1;
    }

    let mut buf: *mut c_char = null_mut();
    let mut len: Py_ssize_t = 0;

    if unsafe { PyBytes_AsStringAndSize(value, addr_of_mut!(buf), addr_of_mut!(len)) } != 0 {
        unsafe { PyErr_Clear() };
        let msg = if le {
            c"bytes_le is not a 16-char bytes object"
        } else {
            c"bytes is not a 16-char bytes object"
        };
        PyTypeError::new_err(msg);
        return -1;
    }

    let ptr = buf as *const u8;

    if le {
        let mut reordered = [0_u8; 16];
        uuid_to_bytes_le_ptr(ptr, reordered.as_mut_ptr());
        bytes_to_hilo(reordered.as_ptr(), hi, lo);
    } else {
        bytes_to_hilo(ptr, hi, lo);
    }
    0
}

pub fn parse_uuid_int(value: *mut PyObject, hi: &mut u64, lo: &mut u64) -> c_int {
    const INT_RANGE_ERR: &CStr = c"int is out of range (need a 128-bit value)";

    if unsafe { PyLong_Check(value) } == 0 {
        PyTypeError::new_err(c"int must be a 128-bit integer");
        return -1;
    }

    #[cfg(all(Py_3_14, not(PyPy)))]
    if crate::python::ffi::is_30bit_layout() {
        use std::ptr::addr_of_mut;

        use pyo3::ffi::{PyLong_Export, PyLong_FreeExport, PyLongExport};

        let mut long_export: PyLongExport = unsafe { std::mem::zeroed() };

        if unsafe { PyLong_Export(value, addr_of_mut!(long_export)) } < 0 {
            unsafe { PyErr_Clear() };
            PyValueError::new_err(INT_RANGE_ERR);
            return -1;
        }

        if long_export.digits.is_null() {
            if long_export.value < 0 {
                PyValueError::new_err(INT_RANGE_ERR);
                return -1;
            }
            *hi = 0;
            *lo = long_export.value.cast_unsigned();
            return 0;
        }

        if long_export.negative != 0 || long_export.ndigits > 5 {
            unsafe { PyLong_FreeExport(addr_of_mut!(long_export)) };
            PyValueError::new_err(INT_RANGE_ERR);
            return -1;
        }

        let ndigits = long_export.ndigits.cast_unsigned();
        let mut digit = [0_u64; 5];
        for (k, slot) in digit.iter_mut().enumerate().take(ndigits) {
            *slot = u64::from(unsafe { *long_export.digits.cast::<u32>().add(k) });
        }

        if digit[4] > 0xFF {
            unsafe { PyLong_FreeExport(addr_of_mut!(long_export)) };
            PyValueError::new_err(INT_RANGE_ERR);
            return -1;
        }

        *lo = digit[0] | (digit[1] << 30) | (digit[2] << 60);
        *hi = (digit[2] >> 4) | (digit[3] << 26) | (digit[4] << 56);

        unsafe { PyLong_FreeExport(addr_of_mut!(long_export)) };
        return 0;
    }

    let mut bytes = [0_u8; 16];
    let rc = {
        #[cfg(Py_3_13)]
        unsafe {
            use std::ffi::c_void;

            use crate::python::ffi::PyLong_AsNativeBytes;

            PyLong_AsNativeBytes(
                value,
                bytes.as_mut_ptr().cast::<c_void>(),
                16,
                pyo3::ffi::Py_ASNATIVEBYTES_BIG_ENDIAN
                    | pyo3::ffi::Py_ASNATIVEBYTES_UNSIGNED_BUFFER
                    | pyo3::ffi::Py_ASNATIVEBYTES_REJECT_NEGATIVE,
            )
        }

        #[cfg(not(Py_3_13))]
        unsafe {
            use std::ffi::c_uchar;

            use pyo3::ffi::PyLongObject;

            use crate::python::ffi::PyLong_AsNativeBytes;

            PyLong_AsNativeBytes(
                value.cast::<PyLongObject>(),
                bytes.as_mut_ptr().cast::<c_uchar>(),
                16,
                0,
                0,
            )
        }
    };

    #[cfg(Py_3_13)]
    let out_of_range = !(0..=16).contains(&rc);
    #[cfg(not(Py_3_13))]
    let out_of_range = rc < 0;
    if out_of_range {
        unsafe { PyErr_Clear() };
        PyValueError::new_err(INT_RANGE_ERR);
        return -1;
    }
    bytes_to_hilo(bytes.as_ptr(), hi, lo);
    0
}

pub fn parse_uuid_fields(value: *mut PyObject, hi: &mut u64, lo: &mut u64) -> c_int {
    static LIMITS: [u64; 6] = [0xFFFF_FFFF, 0xFFFF, 0xFFFF, 0xFF, 0xFF, 0xFFFF_FFFF_FFFF];

    let seq = unsafe { PySequence_Fast(value, c"fields must be a 6-tuple".as_ptr()) };

    if seq.is_null() {
        return -1;
    }

    let size = unsafe { PySequence_Fast_GET_SIZE(seq) };

    if size != 6 {
        unsafe { Py_DECREF(seq) };
        PyValueError::new_err(c"fields is not a 6-tuple");
        return -1;
    }
    let mut parts = [0_u64; 6];

    for i in 0usize..6 {
        let item = {
            #[cfg(not(PyPy))]
            unsafe {
                pyo3::ffi::PySequence_Fast_GET_ITEM(seq, i.cast_signed())
            }

            #[cfg(PyPy)]
            {
                use pyo3::ffi::PySequence_GetItem;

                let item = unsafe { PySequence_GetItem(seq, i.cast_signed()) };
                if item.is_null() {
                    unsafe { Py_DECREF(seq) };
                    return -1;
                }
                item
            }
        };

        let v = unsafe { PyLong_AsUnsignedLongLong(item) };
        unsafe {
            #[cfg(PyPy)]
            Py_DECREF(item);
            if !PyErr_Occurred().is_null() {
                Py_DECREF(seq);
                PyErr_Clear();
                PyTypeError::new_err(c"fields must contain only integers");
                return -1;
            }
        }

        if v > LIMITS[i] {
            unsafe { Py_DECREF(seq) };
            PyValueError::new_err(c"field value out of range");
            return -1;
        }
        parts[i] = v;
    }
    unsafe { Py_DECREF(seq) };
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
