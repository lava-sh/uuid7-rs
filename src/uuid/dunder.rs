use std::ffi::{c_int, c_long};

use pyo3::ffi::{
    Py_EQ, Py_GE, Py_GT, Py_INCREF, Py_LE, Py_LT, Py_NE, Py_NotImplemented, Py_TYPE, Py_hash_t,
    PyBool_FromLong, PyObject,
};

use crate::{
    hex::helpers::fmt_dashed,
    uuid::{class::UUID_PTR, property::with_buf, uuid_obj::UUIDObject},
};

pub extern "C" fn __str__(self_: *mut PyObject) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);

    with_buf(36, |buf| {
        fmt_dashed(obj.hi, obj.lo, buf);
    })
}

pub extern "C" fn __repr__(self_: *mut PyObject) -> *mut PyObject {
    let obj = UUIDObject::from_self(self_);

    with_buf(44, |buf| {
        buf[..6].copy_from_slice(b"UUID('");
        fmt_dashed(obj.hi, obj.lo, &mut buf[6..42]);
        buf[42..44].copy_from_slice(b"')");
    })
}

#[expect(clippy::cast_possible_wrap)]
pub extern "C" fn __hash__(self_: *mut PyObject) -> Py_hash_t {
    let obj = UUIDObject::from_self(self_);
    let h = (obj.hi ^ (obj.time_low()) ^ obj.lo ^ (obj.lo >> 32)) as Py_hash_t;
    if h == -1 { -2 } else { h }
}

pub extern "C" fn __copy__(self_: *mut PyObject, _arg: *mut PyObject) -> *mut PyObject {
    unsafe {
        Py_INCREF(self_);
    }
    self_
}

#[expect(non_upper_case_globals)]
pub extern "C" fn richcompare(a: *mut PyObject, b: *mut PyObject, op: c_int) -> *mut PyObject {
    unsafe {
        if Py_TYPE(a) != UUID_PTR || Py_TYPE(b) != UUID_PTR {
            Py_INCREF(Py_NotImplemented());
            return Py_NotImplemented();
        }
    }

    let a_ = UUIDObject::from_self(a);
    let b_ = UUIDObject::from_self(b);

    let ordering = (a_.hi, a_.lo).cmp(&(b_.hi, b_.lo));

    let result = match op {
        Py_EQ => ordering.is_eq(),
        Py_NE => ordering.is_ne(),
        Py_LT => ordering.is_lt(),
        Py_LE => ordering.is_le(),
        Py_GT => ordering.is_gt(),
        Py_GE => ordering.is_ge(),
        _ => unsafe {
            Py_INCREF(Py_NotImplemented());
            return Py_NotImplemented();
        },
    };
    unsafe { PyBool_FromLong(c_long::from(result)) }
}
