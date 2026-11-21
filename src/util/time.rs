#![allow(dead_code)]

//! Local System Time Utilities

use crate::calendar::{GregorianDate, JalaliDate};
use windows::Win32::System::SystemInformation::GetLocalTime;

pub fn get_current_gregorian() -> GregorianDate {
    unsafe {
        let st = GetLocalTime();
        GregorianDate::new(st.wYear as i32, st.wMonth as u32, st.wDay as u32)
    }
}

pub fn get_current_jalali() -> JalaliDate {
    JalaliDate::from_gregorian(get_current_gregorian())
}

pub fn get_current_system_time() -> (u16, u16, u16) {
    unsafe {
        let st = GetLocalTime();
        (st.wHour, st.wMinute, st.wSecond)
    }
}
