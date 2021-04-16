use std::ffi::c_void;
use crate::sys;
use crate::TypeDescVarType;

#[repr(C)]
struct SafeArrayBound {
    elements: u32,
    lower_bound: i32,
}

/// A COM SAFEARRAY https://docs.microsoft.com/en-us/archive/msdn-magazine/2017/march/introducing-the-safearray-data-structure
#[repr(C)]
pub struct SafeArray(*mut c_void);

impl SafeArray {
    pub fn new(var_type: TypeDescVarType) -> Self {
        let bound = SafeArrayBound {
            elements: 0,
            lower_bound: 0,
        };

        let ptr = unsafe {
            sys::SafeArrayCreate(var_type as _, 1, &bound as *const _ as _)
        };

        Self(ptr)
    }
}

impl Drop for SafeArray {
    fn drop(&mut self) {
        unsafe {
            sys::SafeArrayDestroy(self.0)
        }
    }
}