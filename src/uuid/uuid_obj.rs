use pyo3::ffi::PyObject;

#[repr(C)]
pub struct UUIDObject {
    pub ob_base: PyObject,
    pub hi: u64,
    pub lo: u64,
}

impl UUIDObject {
    #[inline]
    pub fn from_self<'a>(ptr: *mut PyObject) -> &'a Self {
        debug_assert!(!ptr.is_null());
        unsafe { &*ptr.cast::<Self>() }
    }
}
