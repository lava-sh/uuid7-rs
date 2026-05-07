use std::{
    ffi::{c_int, c_void},
    ptr::{addr_of_mut, null_mut},
    sync::OnceLock,
};

use pyo3::ffi::{Py_ssize_t, Py_uintptr_t, PyObject};

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

const PYLONG_BITS_IN_DIGIT: u8 = 30;

// https://docs.python.org/3/c-api/long.html#c.PyLong_GetNativeLayout
#[repr(C)]
struct PyLongLayout {
    bits_per_digit: u8,
    digit_size: u8,
    digits_order: i8,
    digit_endianness: i8,
}

#[repr(C)]
pub struct PyLongExport {
    pub value: i64,
    pub negative: u8,
    pub ndigits: Py_ssize_t,
    pub digits: *const c_void,
    pub _reserved: Py_uintptr_t,
}

extern_libpython! {
    dlls: [
        "python314",
        "python314_d",
        "python315",
        "python315_d",
    ]
    {
        // https://docs.python.org/3/c-api/long.html#c.PyLong_GetNativeLayout
        fn PyLong_GetNativeLayout() -> *const PyLongLayout;

        // https://docs.python.org/3/c-api/long.html#c.PyLongWriter_Create
        fn PyLongWriter_Create(
            negative: c_int,
            ndigits: Py_ssize_t,
            digits: *mut *mut c_void,
        ) -> *mut PyLongWriter;

        // https://docs.python.org/3/c-api/long.html#c.PyLongWriter_Finish
        fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;

        // https://docs.python.org/3/c-api/long.html#c.PyLong_Export
       pub fn PyLong_Export(
            obj: *mut PyObject,
            export_long: *mut PyLongExport,
        ) -> c_int;

        // https://docs.python.org/3/c-api/long.html#c.PyLong_FreeExport
        pub fn PyLong_FreeExport(export_long: *mut PyLongExport);
    }
}

#[must_use]
pub fn is_30bit_layout() -> bool {
    static DIGITS: OnceLock<bool> = OnceLock::new();

    *DIGITS.get_or_init(|| {
        let layout = unsafe { &*PyLong_GetNativeLayout() };
        layout.bits_per_digit == PYLONG_BITS_IN_DIGIT
    })
}

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    const SHIFT: u32 = PYLONG_BITS_IN_DIGIT as u32;
    const MASK: u64 = (1 << SHIFT) - 1;

    if !is_30bit_layout() {
        return super::py_3_13::uuid_int_from_parts(hi, lo);
    }

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
