#[cfg(not(Py_3_13))]
pub use pyo3::ffi::_PyLong_AsByteArray as PyLong_AsNativeBytes;
//
#[cfg(all(Py_3_13, not(PyPy)))]
pub use pyo3::ffi::PyLong_AsNativeBytes;
//
#[cfg(not(PyPy))]
pub use pyo3::ffi::{PySequence_Fast_GET_SIZE, PyTuple_GET_ITEM, PyTuple_GET_SIZE};
//
#[cfg(PyPy)]
pub use pyo3::ffi::{
    PySequence_Size as PySequence_Fast_GET_SIZE, PyTuple_GetItem as PyTuple_GET_ITEM,
    PyTuple_Size as PyTuple_GET_SIZE,
};

#[cfg(not(Py_3_13))]
pub use crate::python::py_3_10_plus::uuid_int_from_parts;
//
#[cfg(all(Py_3_13, not(all(Py_3_14, not(PyPy)))))]
pub use crate::python::py_3_13::uuid_int_from_parts;
//
#[cfg(all(Py_3_14, not(PyPy)))]
pub use crate::python::py_3_14_plus::{is_30bit_layout, uuid_int_from_parts};
