#[cfg(not(Py_3_13))]
use std::os::raw::c_uchar;
use std::{
    ffi::CStr,
    os::raw::{c_int, c_uint, c_void},
    ptr,
    ptr::addr_of_mut,
};

#[cfg(not(Py_3_13))]
use pyo3::ffi::_PyLong_FromByteArray;
#[cfg(Py_3_13)]
use pyo3::ffi::{
    Py_ASNATIVEBYTES_BIG_ENDIAN, Py_ASNATIVEBYTES_UNSIGNED_BUFFER, PyLong_FromUnsignedNativeBytes,
};
#[cfg(not(PyPy))]
use pyo3::ffi::{PyTuple_GET_ITEM, PyTuple_GET_SIZE};
#[cfg(PyPy)]
use pyo3::ffi::{PyTuple_GetItem, PyTuple_Size};
use pyo3::{
    PyErr, PyResult, Python,
    ffi::{
        METH_NOARGS, METH_O, Py_INCREF, Py_None, Py_REFCNT, Py_TPFLAGS_DEFAULT, Py_nb_int,
        Py_ssize_t, Py_tp_dealloc, Py_tp_free, Py_tp_getset, Py_tp_hash, Py_tp_methods, Py_tp_new,
        Py_tp_repr, Py_tp_richcompare, Py_tp_str, PyDict_Next, PyErr_Format, PyErr_SetString,
        PyExc_TypeError, PyGetSetDef, PyMethodDef, PyMethodDefPointer, PyModule_AddObjectRef,
        PyObject, PyObject_Free, PyObject_New, PyObject_TypeCheck, PyType_FromSpec, PyType_Slot,
        PyType_Spec, PyTypeObject, PyUnicode_CompareWithASCIIString,
    },
};

use crate::{
    parse::{parse_uuid_bytes, parse_uuid_fields, parse_uuid_int, parse_uuid_text, uuid_to_bytes},
    uuid::{
        dunder::{__copy__, __hash__, __repr__, __str__, richcompare},
        property::{
            bytes, bytes_le, clock_seq_hi_variant, clock_seq_low, fields, get_clock_seq, hex, int,
            node, time_hi_version, time_low, time_mid, timestamp, urn, version,
        },
        uuid_obj::UUIDObject,
    },
};

pub static mut UUID_PTR: *mut PyTypeObject = ptr::null_mut();
static mut UUID_CACHE: *mut UUIDObject = ptr::null_mut();

pub fn add_obj(m: *mut PyObject, name: &CStr, obj: *mut PyObject) -> PyResult<()> {
    if unsafe { PyModule_AddObjectRef(m, name.as_ptr(), obj) } < 0 {
        return Err(PyErr::fetch(unsafe { Python::assume_attached() }));
    }
    Ok(())
}

#[inline]
pub fn uuid_new(hi: u64, lo: u64) -> *mut UUIDObject {
    let cache = unsafe { UUID_CACHE };

    if !cache.is_null() && unsafe { Py_REFCNT(cache.cast::<PyObject>()) } == 1 {
        unsafe {
            Py_INCREF(cache.cast::<PyObject>());
            (*cache).hi = hi;
            (*cache).lo = lo;
        }
        return cache;
    }

    let obj = unsafe { PyObject_New::<UUIDObject>(UUID_PTR) };

    if obj.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        (*obj).hi = hi;
        (*obj).lo = lo;

        if UUID_CACHE.is_null() {
            Py_INCREF(obj.cast::<PyObject>());
            UUID_CACHE = obj;
        }
    }
    obj
}

pub fn uuid_new_uncached(hi: u64, lo: u64) -> *mut UUIDObject {
    let obj = unsafe { PyObject_New::<UUIDObject>(UUID_PTR) };
    if obj.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        (*obj).hi = hi;
        (*obj).lo = lo;
    }
    obj
}

pub extern "C" fn dealloc(obj: *mut PyObject) {
    unsafe { PyObject_Free(obj.cast::<c_void>()) };
}

pub extern "C" fn uuid_nb_int(self_: *mut PyObject) -> *mut PyObject {
    unsafe {
        let obj = &*(self_ as *const UUIDObject);
        uuid_int_from_parts(obj.hi, obj.lo)
    }
}

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    let mut bytes = [0u8; 16];
    unsafe {
        uuid_to_bytes(hi, lo, bytes.as_mut_ptr());
        #[cfg(Py_3_13)]
        {
            PyLong_FromUnsignedNativeBytes(
                bytes.as_ptr().cast::<c_void>(),
                16,
                Py_ASNATIVEBYTES_BIG_ENDIAN | Py_ASNATIVEBYTES_UNSIGNED_BUFFER,
            )
        }
        #[cfg(not(Py_3_13))]
        {
            _PyLong_FromByteArray(bytes.as_ptr().cast::<c_uchar>(), 16, 0, 0)
        }
    }
}

pub extern "C" fn uuid_type_new(
    tp: *mut PyTypeObject,
    args: *mut PyObject,
    kwargs: *mut PyObject,
) -> *mut PyObject {
    if tp != unsafe { UUID_PTR } {
        return unsafe { (*tp).tp_alloc.unwrap()(tp, 0) };
    }

    #[cfg(not(PyPy))]
    let nargs = unsafe { PyTuple_GET_SIZE(args) };
    #[cfg(PyPy)]
    let nargs = unsafe { PyTuple_Size(args) };

    if nargs > 1 {
        unsafe {
            PyErr_SetString(
                PyExc_TypeError,
                c"UUID() takes at most 1 positional argument".as_ptr(),
            );
        }
        return ptr::null_mut();
    }

    let none = unsafe { Py_None() };

    let mut hex_obj = if nargs == 1 {
        #[cfg(not(PyPy))]
        {
            unsafe { PyTuple_GET_ITEM(args, 0) }
        }
        #[cfg(PyPy)]
        {
            unsafe { PyTuple_GetItem(args, 0) }
        }
    } else {
        none
    };

    let mut bytes_obj = none;
    let mut bytes_le_obj = none;
    let mut fields_obj = none;
    let mut int_obj = none;

    if !kwargs.is_null() {
        let mut pos: Py_ssize_t = 0;
        let mut k: *mut PyObject = ptr::null_mut();
        let mut v: *mut PyObject = ptr::null_mut();
        while unsafe { PyDict_Next(kwargs, addr_of_mut!(pos), addr_of_mut!(k), addr_of_mut!(v)) }
            != 0
        {
            if unsafe { PyUnicode_CompareWithASCIIString(k, c"bytes".as_ptr()) } == 0 {
                bytes_obj = v;
            } else if unsafe { PyUnicode_CompareWithASCIIString(k, c"int".as_ptr()) } == 0 {
                int_obj = v;
            } else if unsafe { PyUnicode_CompareWithASCIIString(k, c"bytes_le".as_ptr()) } == 0 {
                bytes_le_obj = v;
            } else if unsafe { PyUnicode_CompareWithASCIIString(k, c"fields".as_ptr()) } == 0 {
                fields_obj = v;
            } else if unsafe { PyUnicode_CompareWithASCIIString(k, c"hex".as_ptr()) } == 0 {
                if nargs == 1 {
                    unsafe {
                        PyErr_SetString(
                            PyExc_TypeError,
                            c"argument for UUID() given by name ('hex') and position".as_ptr(),
                        );
                    }
                    return ptr::null_mut();
                }
                hex_obj = v;
            } else {
                unsafe {
                    PyErr_Format(
                        PyExc_TypeError,
                        c"UUID() got an unexpected kword argument '%U'".as_ptr(),
                        k,
                    );
                }
                return ptr::null_mut();
            }
        }
    }

    let provided = c_int::from(hex_obj != none)
        + c_int::from(bytes_obj != none)
        + c_int::from(bytes_le_obj != none)
        + c_int::from(fields_obj != none)
        + c_int::from(int_obj != none);

    if provided != 1 {
        unsafe {
            PyErr_SetString(
                PyExc_TypeError,
                c"one of the hex, bytes, bytes_le, fields, or int arguments must be given".as_ptr(),
            );
        }
        return ptr::null_mut();
    }
    unsafe {
        if PyObject_TypeCheck(hex_obj, UUID_PTR) != 0 {
            Py_INCREF(hex_obj);
            return hex_obj;
        }
    }

    let (mut hi, mut lo) = (0u64, 0u64);
    if hex_obj != none {
        if parse_uuid_text(hex_obj, &mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
    } else if bytes_obj != none {
        if parse_uuid_bytes(bytes_obj, false, &mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
    } else if bytes_le_obj != none {
        if parse_uuid_bytes(bytes_le_obj, true, &mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
    } else if fields_obj != none {
        if parse_uuid_fields(fields_obj, &mut hi, &mut lo) != 0 {
            return ptr::null_mut();
        }
    } else if parse_uuid_int(int_obj, &mut hi, &mut lo) != 0 {
        return ptr::null_mut();
    }

    uuid_new(hi, lo).cast::<PyObject>()
}

static mut UUID_METHODS: [PyMethodDef; 3] = [
    PyMethodDef {
        ml_name: c"__copy__".as_ptr(),
        ml_meth: PyMethodDefPointer {
            PyCFunction: __copy__,
        },
        ml_flags: METH_NOARGS,
        ml_doc: ptr::null(),
    },
    PyMethodDef {
        ml_name: c"__deepcopy__".as_ptr(),
        ml_meth: PyMethodDefPointer {
            PyCFunction: __copy__,
        },
        ml_flags: METH_O,
        ml_doc: ptr::null(),
    },
    PyMethodDef::zeroed(),
];

macro_rules! getset {
    ($name:expr, $get:expr) => {
        PyGetSetDef {
            name: $name.as_ptr(),
            get: Some($get),
            set: None,
            doc: ptr::null(),
            closure: ptr::null_mut(),
        }
    };
}

static mut UUID_GETSET: [PyGetSetDef; 17] = [
    getset!(c"bytes", bytes),
    getset!(c"bytes_le", bytes_le),
    getset!(c"clock_seq", get_clock_seq),
    getset!(c"clock_seq_hi_variant", clock_seq_hi_variant),
    getset!(c"clock_seq_low", clock_seq_low),
    getset!(c"fields", fields),
    getset!(c"hex", hex),
    getset!(c"int", int),
    getset!(c"node", node),
    getset!(c"time", timestamp),
    getset!(c"time_hi_version", time_hi_version),
    getset!(c"time_low", time_low),
    getset!(c"time_mid", time_mid),
    getset!(c"timestamp", timestamp),
    getset!(c"urn", urn),
    getset!(c"version", version),
    PyGetSetDef {
        name: ptr::null(),
        get: None,
        set: None,
        doc: ptr::null(),
        closure: ptr::null_mut(),
    },
];

#[allow(non_snake_case)]
pub unsafe fn UUID() -> PyResult<*mut PyObject> {
    let mut slots = [
        PyType_Slot {
            slot: Py_tp_new,
            pfunc: uuid_type_new as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_dealloc,
            pfunc: dealloc as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_repr,
            pfunc: __repr__ as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_str,
            pfunc: __str__ as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_hash,
            pfunc: __hash__ as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_richcompare,
            pfunc: richcompare as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_methods,
            pfunc: addr_of_mut!(UUID_METHODS).cast::<c_void>(),
        },
        PyType_Slot {
            slot: Py_tp_getset,
            pfunc: addr_of_mut!(UUID_GETSET).cast::<c_void>(),
        },
        PyType_Slot {
            slot: Py_nb_int,
            pfunc: uuid_nb_int as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_free,
            pfunc: PyObject_Free as *mut c_void,
        },
        PyType_Slot {
            slot: 0,
            pfunc: ptr::null_mut(),
        },
    ];
    let mut spec = PyType_Spec {
        name: c"uuid7_rs._core._UUID".as_ptr(),
        basicsize: c_int::try_from(size_of::<UUIDObject>())
            .expect("UUIDObject fits PyType_Spec basicsize"),
        itemsize: 0,
        flags: Py_TPFLAGS_DEFAULT as c_uint,
        slots: slots.as_mut_ptr(),
    };
    let tp = unsafe { PyType_FromSpec(addr_of_mut!(spec)) };
    if tp.is_null() {
        return Err(PyErr::fetch(unsafe { Python::assume_attached() }));
    }
    unsafe { UUID_PTR = tp.cast::<PyTypeObject>() };
    Ok(tp)
}
