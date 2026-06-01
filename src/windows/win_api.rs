#![expect(clippy::upper_case_acronyms)]

#[repr(C)]
#[derive(Default)]
pub struct FILETIME {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

// https://learn.microsoft.com/ru-ru/windows/win32/api/realtimeapiset/nf-realtimeapiset-queryinterrupttime
windows_link::link!(
    "api-ms-win-core-realtime-l1-1-1.dll"
    "system"
    fn QueryInterruptTime(lp_interrupt_time: *mut u64)
);

// https://learn.microsoft.com/ru-ru/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimepreciseasfiletime
windows_link::link!(
    "kernel32.dll"
    "system"
    fn GetSystemTimePreciseAsFileTime(lp_system_time_as_file_time: *mut FILETIME)
);
