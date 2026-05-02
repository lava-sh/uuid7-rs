use pyo3::ffi::PyObject;

#[repr(C)]
pub struct UUIDObject {
    pub _ob_base: PyObject,
    pub hi: u64,
    pub lo: u64,
}

impl UUIDObject {
    #[inline]
    pub fn from_self<'a>(ptr: *mut PyObject) -> &'a Self {
        debug_assert!(!ptr.is_null());
        unsafe { &*ptr.cast::<Self>() }
    }

    #[inline]
    pub fn time(&self) -> u64 {
        self.hi >> 16
    }

    #[inline]
    pub fn time_low(&self) -> u64 {
        self.hi >> 32
    }

    #[inline]
    pub fn time_mid(&self) -> u64 {
        self.time() & 0xFFFF
    }

    #[inline]
    pub fn time_hi_version(&self) -> u64 {
        self.hi & 0xFFFF
    }

    #[inline]
    pub fn clock_seq(&self) -> u64 {
        ((self.clock_seq_hi_variant() & 0x3F) << 8) | self.clock_seq_low()
    }

    #[inline]
    pub fn clock_seq_hi_variant(&self) -> u64 {
        self.lo >> 56
    }

    #[inline]
    pub fn clock_seq_low(&self) -> u64 {
        (self.lo >> 48) & 0xFF
    }

    #[inline]
    pub fn node(&self) -> u64 {
        self.lo & 0xFFFF_FFFF_FFFF
    }
}
