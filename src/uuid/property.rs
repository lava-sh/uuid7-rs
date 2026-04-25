use std::{
    os::raw::{c_char, c_ulong, c_void},
    ptr,
};

use pyo3::ffi::{
    Py_ssize_t, PyBytes_FromStringAndSize, PyLong_FromUnsignedLong, PyLong_FromUnsignedLongLong,
    PyObject, PyTuple_New, PyTuple_SET_ITEM, PyUnicode_1BYTE_DATA, PyUnicode_New,
};

use crate::{
    hex::hex::{fmt_dashed, fmt_hex32},
    parse::{uuid_to_bytes, uuid_to_bytes_le},
    uuid::{class::uuid_int_from_parts, uuid_obj::UUIDObject},
};

macro_rules! u64_getter {
    ($name:ident, $expr:expr) => {
        pub extern "C" fn $name(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
            let obj = unsafe { &*(self_ as *const UUIDObject) };
            unsafe { PyLong_FromUnsignedLongLong($expr(obj)) }
        }
    };
}

macro_rules! u32_getter {
    ($name:ident, $expr:expr) => {
        pub extern "C" fn $name(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
            let obj = unsafe { &*(self_ as *const UUIDObject) };
            unsafe { PyLong_FromUnsignedLong($expr(obj) as c_ulong) }
        }
    };
}

// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L370-L387
u64_getter!(timestamp, |obj: &UUIDObject| obj.hi >> 16);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L394-L396
u64_getter!(node, |obj: &UUIDObject| obj.lo & 0xFFFF_FFFF_FFFF);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L350-L352
u32_getter!(time_low, |obj: &UUIDObject| obj.hi >> 32);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L354-L356
u32_getter!(time_mid, |obj: &UUIDObject| (obj.hi >> 16) & 0xFFFF);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L358-L360
u32_getter!(time_hi_version, |obj: &UUIDObject| obj.hi & 0xFFFF);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L362-L364
u32_getter!(clock_seq_hi_variant, |obj: &UUIDObject| obj.lo >> 56);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L366-L368
u32_getter!(clock_seq_low, |obj: &UUIDObject| (obj.lo >> 48) & 0xFF);
// https://github.com/python/cpython/blob/v3.15.0a8/Lib/uuid.py#L417-L421
u32_getter!(version, |obj: &UUIDObject| (obj.hi >> 12) & 0xF);

#[inline]
pub fn with_buf(len: Py_ssize_t, f: impl FnOnce(&mut [u8])) -> *mut PyObject {
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

pub extern "C" fn bytes_le(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };
    let mut bytes = [0u8; 16];
    let mut reordered = [0u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, bytes.as_mut_ptr());
    uuid_to_bytes_le(&bytes, &mut reordered);
    unsafe { PyBytes_FromStringAndSize(reordered.as_ptr().cast::<c_char>(), 16) }
}

pub extern "C" fn int(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };
    uuid_int_from_parts(obj.hi, obj.lo)
}

pub extern "C" fn hex(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };

    with_buf(32, |buf| {
        fmt_hex32(obj.hi, obj.lo, buf);
    })
}

pub extern "C" fn bytes(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };
    let mut bytes = [0u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, bytes.as_mut_ptr());
    unsafe { PyBytes_FromStringAndSize(bytes.as_ptr().cast::<c_char>(), 16) }
}

pub extern "C" fn urn(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };

    with_buf(45, |buf| {
        buf[..9].copy_from_slice(b"urn:uuid:");
        fmt_dashed(obj.hi, obj.lo, &mut buf[9..45]);
    })
}

pub extern "C" fn fields(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };
    let py_tuple = unsafe { PyTuple_New(6) };
    if py_tuple.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        PyTuple_SET_ITEM(
            py_tuple,
            0,
            PyLong_FromUnsignedLong((obj.hi >> 32) as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            1,
            PyLong_FromUnsignedLong(((obj.hi >> 16) & 0xFFFF) as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            2,
            PyLong_FromUnsignedLong((obj.hi & 0xFFFF) as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            3,
            PyLong_FromUnsignedLong((obj.lo >> 56) as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            4,
            PyLong_FromUnsignedLong(((obj.lo >> 48) & 0xFF) as c_ulong),
        );
        PyTuple_SET_ITEM(
            py_tuple,
            5,
            PyLong_FromUnsignedLongLong(obj.lo & 0xFFFF_FFFF_FFFF),
        );
    }
    py_tuple
}

pub extern "C" fn get_clock_seq(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = unsafe { &*(self_ as *const UUIDObject) };
    let hi = ((obj.lo >> 56) & 0x3F) as u32;
    let lo = ((obj.lo >> 48) & 0xFF) as u32;
    unsafe { PyLong_FromUnsignedLong(((hi << 8) | lo) as c_ulong) }
}
