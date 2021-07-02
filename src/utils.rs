use crate::base::*;
use crate::glue;

pub unsafe fn str_from_c_string(c_str: &*const c_char) -> &str {
    let len = glue::strlen(*c_str);
    core::str::from_utf8_unchecked(core::slice::from_raw_parts(*c_str as *const _, len))
}
