use std::ffi::CStr;
use std::os::raw::c_char;

pub unsafe trait StringExt {
    fn from_slice(s: &[c_char]) -> String;
}

unsafe impl StringExt for String {
    fn from_slice(s: &[c_char]) -> String {
        unsafe { CStr::from_ptr(s.as_ptr()).to_string_lossy().to_string() }
    }
}
