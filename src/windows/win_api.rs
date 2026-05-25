#![expect(non_snake_case, clippy::upper_case_acronyms)]

#[repr(C)]
#[derive(Default)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

// https://learn.microsoft.com/ru-ru/windows/win32/api/realtimeapiset/nf-realtimeapiset-queryinterrupttime
windows_link::link!(
    "api-ms-win-core-realtime-l1-1-1.dll"
    "system"
    fn QueryInterruptTime(lpInterruptTime: *mut u64)
);

// https://learn.microsoft.com/ru-ru/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimepreciseasfiletime
windows_link::link!(
    "kernel32.dll"
    "system"
    fn GetSystemTimePreciseAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME)
);
