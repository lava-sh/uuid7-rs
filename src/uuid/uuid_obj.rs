#[repr(C)]
pub struct UUIDObject {
    pub ob_base: pyo3::ffi::PyObject,
    pub hi: u64,
    pub lo: u64,
}

impl UUIDObject {
    #[inline]
    pub fn from_self<'a>(ptr: *mut pyo3::ffi::PyObject) -> &'a Self {
        debug_assert!(!ptr.is_null());
        unsafe { &*ptr.cast::<Self>() }
    }
}