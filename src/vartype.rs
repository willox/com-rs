//! [VARIANT Type Constants](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-oaut/3fe7db9f-5803-4dc4-9d14-5425d3f5461f) equivilent 

/// VARIANT type (VT_*)
/// Not all possible types are specified
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u16)]
pub enum TypeDescVarType {
    /// Nothing!
    Empty = 0,

    /// 2-byte signed integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    I2 = 2,

    /// 4-byte signed integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    I4 = 3,

    /// f32
    /// Contexts: Variants, SafeArrays, TypeDescs
    R4 = 4,

    /// f64
    /// Contexts: Variants, SafeArrays, TypeDescs
    R8 = 5,

    /// Currency. Wtf is it?
    /// Contexts: Variants, SafeArrays, TypeDescs
    Cy = 6,

    /// Date. Wtf is it?
    /// Contexts: Variants, SafeArrays, TypeDescs
    Date = 7,


    /// BSTR
    /// Contexts: Variants, SafeArrays, TypeDescs
    BStr = 8,

    /// IDispatch Ptr
    /// Contexts: Variants, SafeArrays, TypeDescs
    Dispatch = 9,

    /// HRESULT
    /// Contexts: Variants, SafeArrays, TypeDescs
    Error = 10,

    /// VARIANT_BOOL (signed 64-bit, -1 = true, 0 = false)
    /// Contexts: Variants, SafeArrays, TypeDescs
    Bool = 11,

    /// VARIANT inc. VT_BYREF
    /// Contexts: Variants, SafeArrays, TypeDescs
    Variant = 12,

    /// IUnknown Ptr
    /// Contexts: Variants, SafeArrays, TypeDescs
    Unknown = 13,

    /// Decimal. Wtf is it?
    /// Contexts: Variants, SafeArrays, TypeDescs
    Decimal = 14,

    /// 1-byte signed integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    I1 = 16,

    // 1-byte unsigned integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    Ui1 = 17,

    // 2-byte unsigned integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    Ui2 = 18,

    // 4-byte unsigned integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    Ui4 = 19,

    // 8-byte signed integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    I8 = 20,

    // 8-byte unsigned integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    Ui8 = 21,

    // 4-byte signed integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    Int = 22,
    
    // 4-byte unsigned integer
    /// Contexts: Variants, SafeArrays, TypeDescs
    UInt = 23,

    /// Void. Wtf is it?
    /// Contexts: TypeDescs
    Void = 24,

    /// HRESULT. But not error? smh
    /// Contexts: TypeDescs
    HResult = 25,

    /// A _unique_ pointer
    /// Contexts: TypeDescs
    Ptr = 26,

    /// SafeArray
    /// Contexts: TypeDescs
    SafeArray = 27,

    /// fixed-size array
    /// Contexts: TypeDescs
    CArray = 28,

    /// VT_USERDEFINED. Wtf is it?
    /// Contexts: TypeDescs
    UserDefined = 29,

    /// null-terminated string
    /// Contexts: TypeDescs
    LPStr = 30,

    /// null-terminated wide-string
    /// Contexts: TypeDescs
    LPWStr = 31,

    /// A signed integer that is the size of the system's ptr
    /// Contexts: TypeDescs
    IntPtr = 37,

    /// An unsigned integer that is the size of the system's ptr
    /// Contexts: TypeDescs
    UIntPtr = 38,
}