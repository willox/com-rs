//! Everything related to the [ITypeInfo](https://docs.microsoft.com/en-us/windows/win32/api/oaidl/nn-oaidl-itypeinfo) COM interface

use crate::sys::HRESULT;
use crate::interfaces;

use core::ffi::c_void;

interfaces! {
    /// [ITypeInfo](https://docs.microsoft.com/en-us/windows/win32/api/oaidl/nn-oaidl-itypeinfo) COM interface
    #[uuid("00020401-0000-0000-C000-000000000046")]
    pub unsafe interface ITypeInfo : interfaces::IUnknown {
        /// fuk
        pub fn GetTypeAttr(&self, pPTypeAttr: *mut *mut c_void) -> HRESULT;

        /// fuk
        pub fn GetTypeComp(&self, ppTComp: *mut *mut c_void) -> HRESULT;

        /// fuk
        pub fn GetFuncDesc(&self, index: u32, ppFuncDesc: *mut *mut c_void) -> HRESULT;

        /// fuk
        pub fn GetVarDesc(&self, index: u32, ppVarDesc: *mut *mut c_void) -> HRESULT;

        /// fuk
        pub fn GetNames(&self, memid: u32, rgBstrNames: *mut c_void, cMaxNames: u32, pcNames: *mut u32) -> HRESULT;

        fn GetRefTypeOfImplType(&self);
        fn GetImplTypeFlags(&self);
        fn GetIDsOfNames(&self);
        fn Invoke(&self);
        fn GetDocumentation(&self);
        fn GetDllEntry(&self);
        fn GetRefTypeInfo(&self);
        fn AddressOfMember(&self);
        fn CreateInstance(&self);
        fn GetMops(&self);
        fn GetContainingTypeLib(&self);
        fn ReleaseTypeAttr(&self);
        fn ReleaseFuncDesc(&self);
        fn ReleaseVarDesc(&self);
    }
}