use std::{
    ffi::{c_char, c_ulong, c_void},
    ptr,
};

use pyo3::{
    Bound, IntoPyObject, Python,
    ffi::{
        Py_ssize_t, PyBytes_FromStringAndSize, PyLong_FromUnsignedLong,
        PyLong_FromUnsignedLongLong, PyObject,
    },
};

use crate::{
    hex::helpers::{fmt_dashed, fmt_hex32},
    parse::{uuid_to_bytes, uuid_to_bytes_le},
    python::ffi::uuid_int_from_parts,
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

getter!(time, UUIDObject::time, u64);
getter!(node, UUIDObject::node, u64);
getter!(time_low, UUIDObject::time_low);
getter!(time_mid, UUIDObject::time_mid);
getter!(time_hi_version, UUIDObject::time_hi_version);
getter!(clock_seq_hi_variant, UUIDObject::clock_seq_hi_variant);
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
    let mut buf = [0_u8; 16];
    let mut reordered = [0_u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, buf.as_mut_ptr());
    uuid_to_bytes_le(&buf, &mut reordered);
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
    let mut buf = [0_u8; 16];
    uuid_to_bytes(obj.hi, obj.lo, buf.as_mut_ptr());
    unsafe { PyBytes_FromStringAndSize(buf.as_ptr().cast::<c_char>(), 16) }
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
    let py = unsafe { Python::assume_attached() };

    (
        obj.time_low(),
        obj.time_mid(),
        obj.time_hi_version(),
        obj.clock_seq_hi_variant(),
        obj.clock_seq_low(),
        obj.node(),
    )
        .into_pyobject(py)
        .map_or(ptr::null_mut(), Bound::into_ptr)
}

pub extern "C" fn get_clock_seq(self_: *mut PyObject, _: *mut c_void) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);
    unsafe { PyLong_FromUnsignedLongLong(obj.clock_seq()) }
}
