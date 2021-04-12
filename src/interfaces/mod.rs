//! Common COM interfaces including IUknown and IClassFactory

pub mod iclass_factory;
pub mod idispatch;
pub mod itypeinfo;
pub mod iunknown;

#[doc(inline)]
pub use iclass_factory::IClassFactory;
#[doc(inline)]
pub use iunknown::IUnknown;
#[doc(inline)]
pub use idispatch::IDispatch;
#[doc(inline)]
pub use itypeinfo::ITypeInfo;