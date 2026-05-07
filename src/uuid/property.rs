use std::{
    ffi::{c_char, c_ulong, c_void},
    ptr,
};

use pyo3::ffi::{
    Py_ssize_t, PyBytes_FromStringAndSize, PyLong_FromUnsignedLong, PyLong_FromUnsignedLongLong,
    PyObject, PyTuple_New,
};

use crate::{
    hex::helpers::{fmt_dashed, fmt_hex32},
    parse::{uuid_to_bytes, uuid_to_bytes_le},
    python::ffi::{PyTuple_SET_ITEM, uuid_int_from_parts},
    uuid::uuid_obj::UUIDObject,
};

macro_rules! getter {
    ($name:ident, $method:path) => {
        pub extern "C" fn $name(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
            let obj = UUIDObject::from_self(self_);
            unsafe { PyLong_FromUnsignedLong($method(obj) as c_ulong) }
        }
    };
    ($name:ident, $method:path, u64) => {
        pub extern "C" fn $name(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
            let obj = UUIDObject::from_self(self_);
            unsafe { PyLong_FromUnsignedLongLong($method(obj)) }
        }
    };
}

// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L370-L387
getter!(time, UUIDObject::time, u64);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L394-L396
getter!(node, UUIDObject::node, u64);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L350-L352
getter!(time_low, UUIDObject::time_low);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L354-L356
getter!(time_mid, UUIDObject::time_mid);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L358-L360
getter!(time_hi_version, UUIDObject::time_hi_version);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L362-L364
getter!(clock_seq_hi_variant, UUIDObject::clock_seq_hi_variant);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L366-L368
getter!(clock_seq_low, UUIDObject::clock_seq_low);

#[inline]
pub fn with_buf(len: Py_ssize_t, f: impl FnOnce(&mut [u8])) -> *mut PyObject {
    #[cfg(not(PyPy))]
    {
        use pyo3::ffi::{PyUnicode_1BYTE_DATA, PyUnicode_New};

        let py_str = unsafe { PyUnicode_New(len, 127) };
        if py_str.is_null() {
            return ptr::null_mut();
        }

        let buf = unsafe {
            std::slice::from_raw_parts_mut(PyUnicode_1BYTE_DATA(py_str), len.cast_unsigned())
        };

        f(buf);
        py_str
    }
    #[cfg(PyPy)]
    {
        use pyo3::ffi::PyUnicode_FromStringAndSize;

        let mut buf = vec![0_u8; len.cast_unsigned()];
        f(&mut buf);
        unsafe { PyUnicode_FromStringAndSize(buf.as_ptr().cast::<c_char>(), len) }
    }
}

pub extern "C" fn bytes_le(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    let mut bytes = [0_u8; 16];
    let mut reordered = [0_u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, bytes.as_mut_ptr());
    uuid_to_bytes_le(&bytes, &mut reordered);
    unsafe { PyBytes_FromStringAndSize(reordered.as_ptr().cast::<c_char>(), 16) }
}

pub extern "C" fn int(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    uuid_int_from_parts(obj.hi, obj.lo)
}

pub extern "C" fn hex(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);

    with_buf(32, |buf| {
        fmt_hex32(obj.hi, obj.lo, buf);
    })
}

pub extern "C" fn bytes(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    let mut bytes = [0_u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, bytes.as_mut_ptr());
    unsafe { PyBytes_FromStringAndSize(bytes.as_ptr().cast::<c_char>(), 16) }
}

pub extern "C" fn urn(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);

    with_buf(45, |buf| {
        buf[..9].copy_from_slice(b"urn:uuid:");
        fmt_dashed(obj.hi, obj.lo, &mut buf[9..45]);
    })
}

pub extern "C" fn fields(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    let py_tuple = unsafe { PyTuple_New(6) };

    if py_tuple.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        PyTuple_SET_ITEM(
            py_tuple,
            0,
            PyLong_FromUnsignedLong(obj.time_low() as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            1,
            PyLong_FromUnsignedLong(obj.time_mid() as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            2,
            PyLong_FromUnsignedLong(obj.time_hi_version() as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            3,
            PyLong_FromUnsignedLong(obj.clock_seq_hi_variant() as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            4,
            PyLong_FromUnsignedLong(obj.clock_seq_low() as c_ulong),
        );
        PyTuple_SET_ITEM(py_tuple, 5, PyLong_FromUnsignedLongLong(obj.node()));
    }

    py_tuple
}

pub extern "C" fn get_clock_seq(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    unsafe { PyLong_FromUnsignedLongLong(obj.clock_seq()) }
}
