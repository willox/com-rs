use std::ffi::c_void;


/// A COM SAFEARRAY https://docs.microsoft.com/en-us/archive/msdn-magazine/2017/march/introducing-the-safearray-data-structure
#[repr(C)]
pub struct SafeArray(*const c_void);

#[repr(C)]
struct SafeArrayBound {
    elements: u32,
    bounds: i32,
}

#[repr(C)]
struct InternalSafeArray {
    dimensions: u16,
    features: u16,
    elements: u32,
    locks: u32,
    data: *const c_void,
    /* SafeArrayBound bounds[0]; */
}