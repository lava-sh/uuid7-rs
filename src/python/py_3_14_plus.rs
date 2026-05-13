use std::{
    ffi::c_void,
    ptr::{addr_of_mut, null_mut},
    sync::OnceLock,
};

use pyo3::ffi::{PyLong_GetNativeLayout, PyLongWriter_Create, PyLongWriter_Finish, PyObject};

const PYLONG_BITS_IN_DIGIT: u32 = 30;

#[must_use]
pub fn is_30bit_layout() -> bool {
    static DIGITS: OnceLock<bool> = OnceLock::new();

    *DIGITS.get_or_init(|| {
        let layout = unsafe { &*PyLong_GetNativeLayout() };
        layout.bits_per_digit == PYLONG_BITS_IN_DIGIT as u8
    })
}

pub fn uuid_int_from_parts(hi: u64, lo: u64) -> *mut PyObject {
    const SHIFT: u32 = PYLONG_BITS_IN_DIGIT;
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
