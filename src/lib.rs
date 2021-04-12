//! A helper crate for consuming and producing COM interfaces.
//!
//! # Example
//!
//! To work with a COM interface it must first be declared:
//!
//! ```rust,no_run
//! /// Define an IAnimal interface
//! com::interfaces! {
//!     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
//!     pub unsafe interface IAnimal: com::interfaces::IUnknown {
//!         unsafe fn Eat(&self) -> com::sys::HRESULT;
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! To define a COM implementation class:
//!
//! ```rust,no_run
//! # com::interfaces! {
//! #     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
//! #     pub unsafe interface IAnimal: com::interfaces::IUnknown {
//! #         unsafe fn Eat(&self) -> com::sys::HRESULT;
//! #     }
//! # }
//!
//! #[cfg(feature = "production")]
//! com::class! {
//!     pub class BritishShortHairCat: IAnimal {
//!         num_owners: u32,
//!     }
//!
//!     impl IAnimal for BritishShortHairCat {
//!         fn Eat(&self) -> com::sys::HRESULT {
//!             println!("Eating...");
//!             com::sys::NOERROR
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! See the examples directory in the repository for more examples.
//!

#![allow(clippy::transmute_ptr_to_ptr)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

mod abi_transferable;
mod bstring;
mod interface;
pub mod interfaces;
mod param;
#[cfg(windows)]
pub mod runtime;
pub mod sys;
mod vartype;

#[cfg(feature = "production")]
/// Functionality for producing COM classes
pub mod production;

use std::ffi::c_void;

#[doc(inline)]
pub use abi_transferable::AbiTransferable;
#[doc(inline)]
pub use bstring::BString;
#[doc(inline)]
pub use interface::Interface;
#[doc(inline)]
pub use param::Param;
#[doc(inline)]
pub use sys::{CLSID, IID};
#[doc(inline)]
pub use vartype::TypeDescVarType;

com::interfaces! {
    #[uuid("12345678-1234-1234-1234-12345678ABCF")]
    unsafe interface ITest: $IDispatch {
        fn fuck(&self);
    }
}

extern crate self as com;

#[cfg(feature = "production")]
com::class! {
    class Test : ITest($IDispatch) {
        val: u32,
    }

    impl ITest for Test {
        fn fuck(&self) {
            todo!()
        }
    }

}

#[test]
fn dispatch() {
    com::runtime::init_runtime();
    let inst = Test::allocate(23);

    println!("{:?}", inst.val);
    return;
}

/// Declare COM interfaces
///
/// # Example
/// ```rust,no_run
/// /// Define an IAnimal interface
/// com::interfaces! {
///     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
///     pub unsafe interface IAnimal: com::interfaces::IUnknown {
///         unsafe fn Eat(&self) -> com::sys::HRESULT;
///     }
/// }
/// # fn main() {}
/// ```
pub use com_macros::interfaces;

/// Declare COM implementation classes
///
/// # Example
/// ```rust,no_run
/// use com::sys::{HRESULT, NOERROR};
/// # com::interfaces! {
/// #     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
/// #     pub unsafe interface IAnimal: com::interfaces::IUnknown {
/// #         unsafe fn Eat(&self) -> com::sys::HRESULT;
/// #     }
/// # }
///
/// com::class! {
///     pub class BritishShortHairCat: IAnimal {
///         num_owners: u32,
///     }
///
///     impl IAnimal for BritishShortHairCat {
///         fn Eat(&self) -> HRESULT {
///             println!("Eating...");
///             NOERROR
///         }
///     }
/// }
/// # fn main() {}
/// ```
#[cfg(feature = "production")]
pub use com_macros::class;

// this allows for the crate to refer to itself as `com` to keep macros consistent
// whether they are used by some other crate or internally
//#[doc(hidden)]
//extern crate self as com;

// We re-export `alloc` so that we can use `com::alloc::boxed::Box` in generated code,
// for code that uses `#![no_std]`.
#[doc(hidden)]
pub extern crate alloc;
