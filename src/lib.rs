mod hex;
mod parse;
mod python;
mod rng;

#[cfg(unix)]
mod unix;
mod uuid;

#[cfg(windows)]
mod windows;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[pyo3::pymodule(name = "_core")]
mod _core {
    use std::{ptr, ptr::addr_of_mut};

    use pyo3::{
        Bound, PyErr, PyResult,
        ffi::{METH_FASTCALL, METH_KEYWORDS, Py_DECREF, PyMethodDef, PyMethodDefPointer, PyObject},
        prelude::PyModule,
        pyfunction,
    };

    use super::rng;
    use crate::uuid::{
        class::{UUID, add_obj, uuid_new_uncached},
        uuid7::{uuid7, uuid7_int},
    };

    #[pyfunction]
    fn _reseed_rng() {
        rng::reseed();
    }

    #[pymodule_export]
    const _VERSION: &str = env!("CARGO_PKG_VERSION");

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        static mut METHODS: [PyMethodDef; 3] = [
            PyMethodDef {
                ml_name: c"_uuid7".as_ptr(),
                ml_meth: PyMethodDefPointer {
                    PyCFunctionFastWithKeywords: uuid7,
                },
                ml_flags: METH_FASTCALL | METH_KEYWORDS,
                ml_doc: ptr::null(),
            },
            PyMethodDef {
                ml_name: c"_uuid7_int".as_ptr(),
                ml_meth: PyMethodDefPointer {
                    PyCFunctionFastWithKeywords: uuid7_int,
                },
                ml_flags: METH_FASTCALL | METH_KEYWORDS,
                ml_doc: ptr::null(),
            },
            PyMethodDef::zeroed(),
        ];

        let m = module.as_ptr();

        #[cfg(not(PyPy))]
        unsafe {
            pyo3::ffi::PyModule_AddFunctions(m, addr_of_mut!(METHODS).cast::<PyMethodDef>());
        }

        #[cfg(PyPy)]
        {
            use pyo3::ffi::{PyCFunction_NewEx, PyModule_AddObjectRef};

            for method in 0..2 {
                let func = unsafe {
                    PyCFunction_NewEx(
                        addr_of_mut!(METHODS[method]),
                        ptr::null_mut(),
                        ptr::null_mut(),
                    )
                };
                if func.is_null() {
                    return Err(PyErr::fetch(module.py()));
                }

                if unsafe { PyModule_AddObjectRef(m, METHODS[method].ml_name, func) } >= 0 {
                    unsafe {
                        Py_DECREF(func);
                    }
                    continue;
                }

                unsafe {
                    Py_DECREF(func);
                }
                return Err(PyErr::fetch(module.py()));
            }
        }

        add_obj(m, c"_UUID", unsafe { UUID()? })?;

        let nil = uuid_new_uncached(0, 0);
        if nil.is_null() {
            return Err(PyErr::fetch(module.py()));
        }
        add_obj(m, c"_NIL", nil.cast::<PyObject>())?;
        unsafe {
            Py_DECREF(nil.cast::<PyObject>());
        }

        let max = uuid_new_uncached(u64::MAX, u64::MAX);
        if max.is_null() {
            return Err(PyErr::fetch(module.py()));
        }
        add_obj(m, c"_MAX", max.cast::<PyObject>())?;
        unsafe {
            Py_DECREF(max.cast::<PyObject>());
        }
        Ok(())
    }
}
