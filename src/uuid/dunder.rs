use std::{ffi::c_int, ptr};

use pyo3::ffi::{
    Py_EQ, Py_False, Py_GE, Py_GT, Py_INCREF, Py_LE, Py_LT, Py_NE, Py_NewRef, Py_NotImplemented,
    Py_TYPE, Py_True, Py_hash_t, PyErr_Format, PyExc_SystemError, PyObject,
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
pub extern "C" fn richcompare(
    self_: *mut PyObject,
    other: *mut PyObject,
    op: c_int,
) -> *mut PyObject {
    unsafe {
        if Py_TYPE(self_) != UUID_PTR || Py_TYPE(other) != UUID_PTR {
            Py_INCREF(Py_NotImplemented());
            return Py_NotImplemented();
        }
    }

    let self_ = UUIDObject::from_self(self_);
    let other = UUIDObject::from_self(other);

    let ordering = (self_.hi, self_.lo).cmp(&(other.hi, other.lo));

    let cmp = match op {
        Py_LT => ordering.is_lt(),
        Py_LE => ordering.is_le(),
        Py_EQ => ordering.is_eq(),
        Py_NE => ordering.is_ne(),
        Py_GT => ordering.is_gt(),
        Py_GE => ordering.is_ge(),
        unrecognized => {
            unsafe {
                PyErr_Format(
                    PyExc_SystemError,
                    c"unrecognized richcompare opcode %d".as_ptr(),
                    unrecognized,
                );
            }
            return ptr::null_mut();
        }
    };

    if cmp {
        unsafe { Py_NewRef(Py_True()) }
    } else {
        unsafe { Py_NewRef(Py_False()) }
    }
}
