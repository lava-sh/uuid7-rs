#[repr(C)]
pub struct UUIDObject {
    pub ob_base: pyo3::ffi::PyObject,
    pub hi: u64,
    pub lo: u64,
}
