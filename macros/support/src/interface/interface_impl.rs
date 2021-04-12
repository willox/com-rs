use super::Interface;
use crate::interface::{iid, vtable, InterfaceMethodKind};
use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

fn method_data(interface: &Interface) -> TokenStream {
    let reserved_dispatch_ids = {
        let mut set = HashSet::new();

        for method in &interface.methods {
            if let Some(id) = method.dispatch_id {
                set.insert(id);
            }
        }

        set
    };

    let mut param_buffer = vec![];
    let mut method_buffer = vec![];

    let mut current_dispatch_id = 0;

    for method in &interface.methods {
        current_dispatch_id += 1;
        while reserved_dispatch_ids.contains(&current_dispatch_id) {
            current_dispatch_id += 1;
        }

        let param_offset = param_buffer.len();
        let param_count = method.args.len() as u32;

        for param in &method.args {
            let ty = param.ty.clone();

            param_buffer.push(quote! {
                ::com::interfaces::idispatch::ParamData {
                    name: std::ptr::null(),
                    var_type: <#ty as ::com::AbiTransferable>::VAR_TYPE,
                },
            });
        }

        let flags: u16 = match method.kind {
            InterfaceMethodKind::Method => 1,
            InterfaceMethodKind::PropertyGet => 2,
            InterfaceMethodKind::PropertySet => 4,
        };

        let params = if param_count > 0 {
            quote!(&params[#param_offset] as *const _)
        } else {
            quote!(std::ptr::null())
        };

        let ret_type = match &method.ret {
            syn::ReturnType::Default => quote!(::com::TypeDescVarType::Empty),
            syn::ReturnType::Type(_, ty) => quote!(<#ty as ::com::AbiTransferable>::VAR_TYPE),
        };

        let dispatch_id = match method.dispatch_id {
            Some(id) => id,
            _ => current_dispatch_id,
        };

        method_buffer.push(quote! {
            ::com::interfaces::idispatch::MethodData {
                name: std::ptr::null(),
                params: #params,
                dispatch_id: ::com::interfaces::idispatch::DispatchId(#dispatch_id),
                method_id: 0, // Populated later
                calling_convention: ::com::interfaces::idispatch::CallingConvention::StdCall,
                params_count: #param_count,
                flags: #flags,
                return_type: #ret_type,
            },
        });
    }

    let param_buffer_len = param_buffer.len();
    let method_buffer_len = method_buffer.len();

    quote! {
        fn static_method_data() -> &'static [::com::interfaces::idispatch::MethodData] {
            static params: [::com::interfaces::idispatch::ParamData; #param_buffer_len] = [
                #(#param_buffer)*
            ];
            static methods: [::com::interfaces::idispatch::MethodData; #method_buffer_len] = [
                #(#method_buffer)*
            ];
            &methods[..]
        }
    }
}

pub fn generate(interface: &Interface) -> TokenStream {
    let interface_ident = &interface.name;
    let vtable_ident = vtable::ident(&interface_ident.to_string());
    let iid_ident = iid::ident(interface_ident);
    let parent = if let Some(p) = &interface.parent {
        quote! { #p }
    } else {
        quote! { #interface_ident }
    };

    let method_data = method_data(interface);

    quote! {
        unsafe impl com::Interface for #interface_ident {
            type VTable = #vtable_ident;
            type Super = #parent;
            const IID: com::sys::IID = #iid_ident;
            #method_data
        }
    }
}
