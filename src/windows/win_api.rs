#![expect(non_snake_case, clippy::upper_case_acronyms)]

#[repr(C)]
#[derive(Default)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetSystemTimePreciseAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
}

#[link(name = "mincore")]
unsafe extern "system" {
    pub fn QueryInterruptTime(lpInterruptTime: *mut u64);
}
