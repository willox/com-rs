use super::{class::Interface, Class};
use proc_macro2::TokenStream;
use quote::quote;

/// Generates a function used to instantiate the COM class
pub fn generate(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let std_dispatch_default = if class.requires_dispatch {
        Some(quote! {
            __std_dispatch: None,
        })
    } else {
        None
    };

    let std_dispatch_init = if class.requires_dispatch {
        let chain = class.interfaces.iter().find(|x| x.needs_dispatch_imp).unwrap();
        let interface = chain.path.clone();

        Some(quote! {
            unsafe {
                let mut type_info: Option<::com::interfaces::ITypeInfo> = None;
                let res = ::com::sys::CreateDispTypeInfo(
                    interface_data_ptr as *const _,
                    0,
                    &mut type_info as *mut _ as _,
                );
                assert_eq!(res, ::com::sys::S_OK,);
                let mut type_info = type_info.unwrap();

                let mut our_unkown = instance.query_interface::<::com::interfaces::IUnknown>().unwrap();
                let mut our_dispatch = instance.query_interface::<#interface>().unwrap();

                let res  = ::com::sys::CreateStdDispatch(
                    std::mem::transmute(our_unkown),
                    std::mem::transmute(our_dispatch),
                    std::mem::transmute(type_info),
                    &mut instance.__std_dispatch as *mut _ as _,
                );
                assert_eq!(res, ::com::sys::S_OK);
            }
        })
    } else {
        None
    };

    let parameters = &class.fields;
    let user_fields = class.fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name
        }
    });

    let interface_inits = gen_vpointer_inits(class);

    let interface_data = if class.requires_dispatch {
        let chain = class.interfaces.iter().find(|x| x.needs_dispatch_imp).unwrap();
        Some(chain.to_interface_data_ptr_tokens())
    } else {
        None
    };

    let ref_count_ident = crate::utils::ref_count_ident();

    let interfaces = &class.interfaces;
    let interface_fields = gen_allocate_interface_fields(interfaces);

    quote! {
        /// Allocate the class casting it to the supplied interface
        ///
        /// This allocates the class on the heap and pins it. This is because COM classes
        /// must have a stable location in memory. Once a COM class is instantiated somewhere
        /// it must stay there.
        #vis fn allocate(#(#parameters),*) -> ::com::production::ClassAllocation<Self> {
            #interface_inits
            #interface_data
            let instance = #name {
                #interface_fields
                #ref_count_ident: ::core::cell::Cell::new(1),
                #std_dispatch_default
                #(#user_fields),*
            };
            let mut instance = ::com::alloc::boxed::Box::pin(instance);
            #std_dispatch_init
            ::com::production::ClassAllocation::new(instance)
        }
    }
}

// Generate the vptr field idents needed in the instantiation syntax of the COM struct.
fn gen_allocate_interface_fields(interface_idents: &[Interface]) -> TokenStream {
    let base_fields = interface_idents
        .iter()
        .enumerate()
        .map(|(index, _)| quote::format_ident!("__{}", index));

    quote!(#(#base_fields,)*)
}

// Initialise VTables with the correct adjustor thunks
fn gen_vpointer_inits(class: &Class) -> TokenStream {
    let interface_inits = class.interfaces
        .iter()
        .enumerate()
        .map(move |(index,  interface)| {
            let interface = interface.to_initialized_vtable_tokens(class, index);
            let vptr_field_ident = quote::format_ident!("__{}", index);
            quote! {
                let #vptr_field_ident = unsafe {
                    ::core::ptr::NonNull::new_unchecked(
                        ::com::alloc::boxed::Box::into_raw(::com::alloc::boxed::Box::new(#interface)),
                    )
                };
            }
        });

    quote!(#(#interface_inits)*)
}
