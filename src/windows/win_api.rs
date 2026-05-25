#![expect(non_snake_case, clippy::upper_case_acronyms)]

#[repr(C)]
#[derive(Default)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

windows_link::link!(
    "mincore.dll"
    "system"
    fn QueryInterruptTime(lpInterruptTime: *mut u64)
);

windows_link::link!(
    "kernel32.dll"
    "system"
    fn GetSystemTimePreciseAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME)
);
