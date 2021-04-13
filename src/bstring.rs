use std::convert::TryFrom;
use crate::{AbiTransferable, TypeDescVarType, sys};

/// A COM BSTR https://docs.microsoft.com/en-us/previous-versions/windows/desktop/automat/bstr
#[repr(C)]
pub struct BString(*mut u16);

impl BString {
    fn new(data: &[u16]) -> Self {
        let len = u32::try_from(data.len()).unwrap();

        let alloc = unsafe {
            sys::SysAllocStringLen(data.as_ptr() as _, len)
        };

        Self(alloc as _)
    }

    fn len(&self) -> usize {
        let len = unsafe {
            sys::SysStringLen(self.0 as _)
        };

        usize::try_from(len).unwrap()
    }
}

impl Drop for BString {
    fn drop(&mut self) {
        unsafe {
            sys::SysFreeString(self.0 as _);
        }
    }
}

unsafe impl AbiTransferable for BString {
    type Abi = *mut u16;
    const VAR_TYPE: TypeDescVarType = TypeDescVarType::BStr;
    fn get_abi(&self) -> Self::Abi {
        // TODO
        let copy = self.clone();
        let ptr = copy.0;
        std::mem::forget(copy);
        ptr
    }
    fn set_abi(&mut self) -> *mut Self::Abi {
        self.0 as *mut Self::Abi
    }
}

impl AsRef<[u16]> for BString {
    fn as_ref(&self) -> &[u16] {
        unsafe {
            std::slice::from_raw_parts(self.0, self.len())
        }
    }
}

impl From<&str> for BString {
    fn from(str: &str) -> Self {
        let utf16_data: Vec<u16> = str.encode_utf16().collect();
        Self::new(&utf16_data)        
    }
}

impl TryFrom<&BString> for String {
    type Error = std::string::FromUtf16Error;

    fn try_from(bstr: &BString) -> Result<Self, Self::Error> {
        Self::from_utf16(bstr.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_nulls() {
        let basic_string = BString::from("Hello\0World!");
        let rust_string = String::try_from(basic_string).unwrap();
        assert_eq!(rust_string, "Hello\0World!");
    }

}
