use std::ptr;

use pyo3::ffi::{
    Py_None, Py_ssize_t, PyErr_Format, PyExc_TypeError, PyObject, PyUnicode_Check,
    PyUnicode_CompareWithASCIIString,
};

use crate::{
    parse::parse_u64_arg,
    python::{
        exceptions::{PyTypeError, PyValueError},
        ffi::{PyTuple_GET_ITEM, PyTuple_GET_SIZE},
    },
    rng::{
        build_timestamp_ms, build_uuid7_default, build_uuid7_default_secure, build_uuid7_with_args,
        build_uuid7_with_args_secure,
    },
    uuid::class::uuid_new,
};

pub extern "C" fn uuid7(
    _self: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    const MAX_NANOS: u64 = 1_000_000_000;

    let nkw = if kwnames.is_null() {
        0
    } else {
        unsafe { PyTuple_GET_SIZE(kwnames) }
    };

    if nargs == 0 && nkw == 0 {
        let (mut hi, mut lo) = (0u64, 0u64);
        if build_uuid7_default(&mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
        return uuid_new(hi, lo).cast::<PyObject>();
    }

    if nargs > 3 {
        PyTypeError::new_err(c"uuid7() takes at most 3 positional arguments");
        return ptr::null_mut();
    }

    let none = unsafe { Py_None() };

    let mut ts = none;
    let mut nanos = none;
    let mut mode = none;

    match nargs {
        0 => {}
        1 => {
            ts = unsafe { *args.add(0) };
        }
        2 => {
            ts = unsafe { *args.add(0) };
            nanos = unsafe { *args.add(1) };
        }
        3 => {
            ts = unsafe { *args.add(0) };
            nanos = unsafe { *args.add(1) };
            mode = unsafe { *args.add(2) };
        }
        _ => unreachable!(),
    }

    for i in 0..nkw {
        let k = unsafe { PyTuple_GET_ITEM(kwnames, i) };
        let v = unsafe { *args.offset(nargs + i) };

        match () {
            () if unsafe { PyUnicode_CompareWithASCIIString(k, c"timestamp".as_ptr()) } == 0 => {
                ts = v;
            }
            () if unsafe { PyUnicode_CompareWithASCIIString(k, c"nanos".as_ptr()) } == 0 => {
                nanos = v;
            }
            () if unsafe { PyUnicode_CompareWithASCIIString(k, c"mode".as_ptr()) } == 0 => {
                mode = v;
            }
            () => {
                unsafe {
                    PyErr_Format(
                        PyExc_TypeError,
                        c"uuid7() got an unexpected keyword argument '%U'".as_ptr(),
                        k,
                    );
                }
                return ptr::null_mut();
            }
        }
    }

    let secure = if mode == none || mode.is_null() {
        false
    } else {
        if unsafe { PyUnicode_Check(mode) } == 0 {
            PyTypeError::new_err(c"mode must be 'fast', 'secure', or None");
            return ptr::null_mut();
        }

        match () {
            () if unsafe { PyUnicode_CompareWithASCIIString(mode, c"fast".as_ptr()) } == 0 => false,
            () if unsafe { PyUnicode_CompareWithASCIIString(mode, c"secure".as_ptr()) } == 0 => {
                true
            }
            () => {
                PyValueError::new_err(c"mode must be 'fast' or 'secure'");
                return ptr::null_mut();
            }
        }
    };

    if secure && ts == none && nanos == none {
        let (mut hi, mut lo) = (0u64, 0u64);

        if build_uuid7_default_secure(&mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
        return uuid_new(hi, lo).cast::<PyObject>();
    }

    let (has_ts, ts_s) = parse_u64_arg(ts, c"timestamp".as_ptr());

    if has_ts < 0 {
        return ptr::null_mut();
    }

    let (has_nanos, nanos) = parse_u64_arg(nanos, c"nanos".as_ptr());
    if has_nanos < 0 {
        return ptr::null_mut();
    }

    if has_nanos > 0 && nanos >= MAX_NANOS {
        PyValueError::new_err(c"nanos must be in range 0..999999999");
        return ptr::null_mut();
    }

    let timestamp_ms = match build_timestamp_ms(ts_s, has_ts > 0, nanos, has_nanos > 0) {
        Ok(v) => v,
        Err(()) => return ptr::null_mut(),
    };

    let (mut hi, mut lo) = (0u64, 0u64);

    let mode = if secure {
        build_uuid7_with_args_secure(
            timestamp_ms,
            has_ts > 0,
            nanos,
            has_nanos > 0,
            &mut hi,
            &mut lo,
        )
    } else {
        build_uuid7_with_args(
            timestamp_ms,
            has_ts > 0,
            nanos,
            has_nanos > 0,
            &mut hi,
            &mut lo,
        )
    };

    if mode != 0 {
        return ptr::null_mut();
    }

    uuid_new(hi, lo).cast::<PyObject>()
}
