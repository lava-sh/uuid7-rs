use std::{
    ffi::{c_int, c_void},
    ptr::{addr_of_mut, null_mut},
};

use pyo3::ffi::{Py_ssize_t, PyObject};

macro_rules! extern_libpython {
        (dlls: [$($dll:literal),* $(,)?] { $($body:item)* }) => {
            $(
                #[cfg_attr(
                    all(windows, target_arch = "x86", pyo3_dll = $dll),
                    link(name = $dll, kind = "raw-dylib", import_name_type = "undecorated")
                )]
                #[cfg_attr(
                    all(windows, not(target_arch = "x86"), pyo3_dll = $dll),
                    link(name = $dll, kind = "raw-dylib")
                )]
            )*
            unsafe extern "C" {
                $($body)*
            }
        };
    }

#[repr(C)]
struct PyLongWriter([u8; 0]);

extern_libpython! {
    dlls: [
        "python314",
        "python314_d",
        "python315",
        "python315_d",
    ]
    {
        // https://docs.python.org/3/c-api/long.html#c.PyLongWriter_Create
        fn PyLongWriter_Create(
            negative: c_int,
            ndigits: Py_ssize_t,
            digits: *mut *mut c_void,
        ) -> *mut PyLongWriter;

        fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;
    }
}

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    const SHIFT: u32 = 30;
    const MASK: u64 = (1 << SHIFT) - 1;

    let mut ptr: *mut c_void = null_mut();

    let writer = unsafe { PyLongWriter_Create(0, 5, addr_of_mut!(ptr)) };

    if writer.is_null() {
        return null_mut();
    }

    let digit = ptr.cast::<u32>();
    unsafe {
        digit.write((lo & MASK) as u32);
        digit.add(1).write(((lo >> 30) & MASK) as u32);
        digit.add(2).write((((lo >> 60) | (hi << 4)) & MASK) as u32);
        digit.add(3).write(((hi >> 26) & MASK) as u32);
        digit.add(4).write((hi >> 56) as u32);
        PyLongWriter_Finish(writer)
    }
}
