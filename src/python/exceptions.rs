use std::ffi::CStr;

use pyo3::ffi::{PyErr_SetString, PyExc_OSError, PyExc_TypeError, PyExc_ValueError};

macro_rules! exception {
    ($name:ident, $exc:ident) => {
        pub struct $name;

        impl $name {
            #[inline]
            pub fn new_err(msg: &CStr) {
                unsafe {
                    PyErr_SetString($exc, msg.as_ptr());
                }
            }
        }
    };
}

exception!(PyOSError, PyExc_OSError);
exception!(PyTypeError, PyExc_TypeError);
exception!(PyValueError, PyExc_ValueError);
