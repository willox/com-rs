//! Everything related to the [IDispatch](https://docs.microsoft.com/en-us/windows/win32/api/oaidl/nn-oaidl-idispatch) COM interface

use crate::sys::HRESULT;
use crate::interfaces;
use crate::vartype::TypeDescVarType;

/// COM DISPID
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DispatchId(pub i32);

/// COM CALLCONV
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum CallingConvention {
    /// The only calling convention I care about
    StdCall = 4,
}

/// oleauto.h tagPARAMDATA
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ParamData {
    /// name of this parameter
    pub name: *const u16,

    /// type of this parameter
    pub var_type: TypeDescVarType,
}

unsafe impl Sync for ParamData {}

/// oleauto.h tagMETHODDATA
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MethodData {
    /// name of this method
    pub name: *const u16,

    /// parameters for this method
    pub params: *const ParamData,

    /// dispatch id for the method, this is part of the API
    pub dispatch_id: DispatchId,

    /// index into IUnknown's vtable for this method
    pub method_id: u32,

    /// calling convention for this method
    pub calling_convention: CallingConvention,

    /// number of parameters for this method
    pub params_count: u32,

    /// IDispatch::Invoke flags (TODO)
    pub flags: u16,

    /// return-type of this method
    pub return_type: TypeDescVarType,
}

unsafe impl Sync for MethodData {}

/// oleauto.h tagINTERFACEDATA
#[repr(C)]
pub struct InterfaceData {
    pub methods: *const MethodData,
    pub method_count: u32,
}

unsafe impl Sync for InterfaceData {}

interfaces! {
    /// [IDispatch](https://docs.microsoft.com/en-us/windows/win32/api/oaidl/nn-oaidl-idispatch) COM interface
    #[uuid("00020400-0000-0000-C000-000000000046")]
    pub unsafe interface IDispatch : interfaces::IUnknown
    {
        /// COM stuff
        pub fn GetTypeInfoCount(&self);

        /// COM stuff
        pub fn GetTypeInfo(&self);

        /// COM stuff
        pub fn GetIDsOfNames(&self, id: *const com::sys::IID, names: *const *const u16, count: u32, lcid: u32, out: *mut u32) -> HRESULT;

        /// COM stuff
        pub fn Invoke(
            &self,
            disp_id: u32,
            riid: *const com::sys::IID,
            lcid: u32,
            flags: u16,
            params: *const u32,
            result: *mut u32,
            excep_info: *mut u32,
            arg_err: *mut u32
        ) -> HRESULT;
    }
}