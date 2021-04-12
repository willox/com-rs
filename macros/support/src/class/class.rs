use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use syn::parse::ParseBuffer;

#[derive(Debug)]
pub struct Class {
    pub name: Ident,
    pub has_class_factory: bool,
    pub docs: Vec<syn::Attribute>,
    pub visibility: syn::Visibility,
    pub interfaces: Vec<Interface>,
    pub requires_dispatch: bool,
    pub methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>>,
    pub fields: Vec<syn::Field>,
    pub impl_debug: bool,
}

impl Class {
    pub fn to_tokens(&self) -> TokenStream {
        let mut out: Vec<TokenStream> = Vec::new();
        out.push(self.to_struct_tokens());
        out.push(self.to_class_trait_impl_tokens());
        out.push(super::class_factory::generate(self));

        TokenStream::from_iter(out)
    }

    /// Get the paths of all interfaces including parent interfaces
    fn interfaces_paths<'a>(&'a self) -> HashSet<&'a syn::Path> {
        fn get_interface<'a>(interface: &'a Interface, result: &mut HashSet<&'a syn::Path>) {
            result.insert(&interface.path);
            if let Some(i) = &interface.parent {
                get_interface(i, result);
            }
        }
        let mut result = HashSet::new();
        for i in &self.interfaces {
            get_interface(i, &mut result)
        }
        result
    }

    /// Parse the class macro syntax (without the `impl`s)
    fn parse_class(
        input: syn::parse::ParseStream,
        docs: Vec<syn::Attribute>,
        has_class_factory: bool,
    ) -> syn::Result<Self> {
        let mut interfaces: Vec<Interface> = Vec::new();
        let mut encountered_dispatch = false;
        let visibility = input.parse::<syn::Visibility>()?;

        let _ = input.parse::<keywords::class>()?;
        let name = input.parse::<Ident>()?;
        let _ = input.parse::<syn::Token!(:)>()?;

        while !input.peek(syn::token::Brace) {
            let path = input.parse::<syn::Path>()?;
            let interface = Interface {
                path: path.clone(),
                parent: None,
                needs_dispatch_imp: false,
            };
            if interfaces.iter().any(|i| i.path == path) {
                return Err(syn::Error::new(path.span(), "interface was redefined"));
            }
            interfaces.push(interface);

            let mut current = interfaces.last_mut().unwrap();
            fn parse_parens(buffer: &ParseBuffer, current: &mut Interface) -> syn::Result<bool> {
                let mut is_dispatched = false;

                while buffer.peek(syn::token::Paren) {
                    let contents;
                    syn::parenthesized!(contents in buffer);

                    // Special names
                    let path = if contents.peek(syn::Token![$]) {
                        contents.parse::<syn::Token![$]>()?;

                        match contents.parse::<Ident>()? {
                            ident if ident == "IDispatch" => {
                                is_dispatched = true;
                                syn::parse_quote!(::com::interfaces::IDispatch)
                            }
        
                            other => {
                                return Err(syn::Error::new(
                                    other.span(),
                                    format!("Unknown built-in interface `{}`", other),
                                ));
                            }
                        }
                    } else {
                        contents.parse::<syn::Path>()?
                    };

                    let parent = Interface {
                        path,
                        parent: None,
                        needs_dispatch_imp: false,
                    };
                    current.parent = Some(Box::new(parent));
                    if !contents.is_empty() {
                        is_dispatched = parse_parens(&contents, current.parent.as_mut().unwrap().as_mut())? || is_dispatched;
                    }
                }

                Ok(is_dispatched)
            }

            if parse_parens(&input, &mut current)? {
                if encountered_dispatch {
                    // Something already inherited IDispatch - this isn't allowed
                    return Err(syn::Error::new(
                        path.span(),
                        "IDispatch found in multiple interface inheritance chains",
                    ));
                } 

                encountered_dispatch = true;
                current.needs_dispatch_imp = true;
            }

            if !input.peek(syn::token::Brace) {
                let _ = input.parse::<syn::Token!(,)>()?;
            }
        }
        let fields;
        syn::braced!(fields in input);
        let fields =
            syn::punctuated::Punctuated::<syn::Field, syn::Token!(,)>::parse_terminated_with(
                &fields,
                syn::Field::parse_named,
            )?;
        let fields = fields.into_iter().collect();

        Ok(Class {
            name,
            has_class_factory,
            docs,
            visibility,
            interfaces,
            requires_dispatch: encountered_dispatch,
            methods: HashMap::new(),
            fields,
            impl_debug: false,
        })
    }

    /// The COM class object struct and `impl`
    ///
    /// Structure of the object:
    /// ```rust
    /// pub struct ClassName {
    ///     // ..interface vpointers..
    ///     // ..ref count..
    ///     // ..user defined fields..
    /// }
    /// ```
    pub fn to_struct_tokens(&self) -> TokenStream {
        let name = &self.name;
        let vis = &self.visibility;

        let interfaces = &self.interfaces;
        let interface_fields = interfaces.iter().enumerate().map(|(index, interface)| {
            let interface_name = &interface.path;
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                #field_ident: ::core::ptr::NonNull<<#interface_name as ::com::Interface>::VTable>
            }
        });
        let ref_count_ident = crate::utils::ref_count_ident();

        let std_dispatch = if self.requires_dispatch {
            Some(quote! {
                __std_dispatch: Option<com::interfaces::IUnknown>,
            })
        } else {
            None
        };

        let user_fields = &self.fields;
        let docs = &self.docs;
        let methods = self.methods.values().flatten().map(|m| {
            quote! {
                #[allow(non_snake_case)]
                #m
            }
        });

        let iunknown = super::iunknown_impl::IUnknown::new();
        let add_ref = iunknown.to_add_ref_tokens();
        let query_interface = iunknown.to_query_interface_tokens(interfaces, self.requires_dispatch);
        let constructor = super::class_constructor::generate(self);
        let interface_drops = interfaces.iter().enumerate().map(|(index, _)| {
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                let _ = ::com::alloc::boxed::Box::from_raw(self.#field_ident.as_ptr());
            }
        });
        let debug = self.debug();
        let safe_query_interface = self.safe_query_interface();

        quote! {
            #(#docs)*
            #[repr(C)]
            #vis struct #name {
                #(#interface_fields,)*
                #ref_count_ident: ::core::cell::Cell<u32>,
                #std_dispatch
                #(#user_fields),*
            }
            impl #name {
                #constructor
                #(#methods)*
                #add_ref
                #query_interface
                #safe_query_interface
            }
            #debug
            impl ::core::ops::Drop for #name {
                fn drop(&mut self) {
                    unsafe {
                        #(#interface_drops)*
                    }
                }
            }
        }
    }

    pub fn to_class_trait_impl_tokens(&self) -> TokenStream {
        let name = &self.name;
        let factory = if self.has_class_factory {
            let ident = crate::utils::class_factory_ident(name);
            quote! { #ident }
        } else {
            quote! { () }
        };
        let ref_count_ident = crate::utils::ref_count_ident();

        quote! {
            unsafe impl com::production::Class for #name {
                type Factory = #factory;

                fn dec_ref_count(&self) -> u32 {
                    let count = self.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
                    self.#ref_count_ident.set(count);
                    count
                }
            }
        }
    }

    fn debug(&self) -> TokenStream {
        if !self.impl_debug {
            return TokenStream::new();
        }

        let name = &self.name;
        let fields = self.fields.iter().map(|f| {
            let name = f.ident.as_ref().unwrap();
            quote! {
                .field(::core::stringify!(#name), &self.#name)
            }
        });

        quote! {
            impl ::core::fmt::Debug for #name {
                 fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(::core::stringify!(#name))
                        #(#fields)*
                        .finish()
                    }
            }
        }
    }

    fn safe_query_interface(&self) -> TokenStream {
        quote! {
            pub fn query_interface<T: ::com::Interface>(self: &::core::pin::Pin<::com::alloc::boxed::Box<Self>>) -> Option<T> {
                let mut result = None;
                let hr = unsafe { self.QueryInterface(&T::IID, &mut result as *mut _ as _) };

                if ::com::sys::FAILED(hr) {
                    return None;
                }
                debug_assert!(result.is_some(), "Successful call to query_interface yielded a null pointer");
                result
            }
        }
    }
}

impl syn::parse::Parse for Class {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class = None;
        let mut methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>> = HashMap::new();
        let mut impl_debug = false;
        while !input.is_empty() {
            let attributes = input.call(syn::Attribute::parse_outer)?;
            let mut docs = Vec::with_capacity(attributes.len());
            let mut has_class_factory = true;
            for attr in attributes {
                if attr.path.is_ident("doc") {
                    docs.push(attr)
                } else if attr.path.is_ident("no_class_factory") {
                    has_class_factory = false;
                } else if attr.path.is_ident("derive") {
                    parse_derive_debug(&attr)?;
                    impl_debug = true;
                } else {
                    return Err(syn::Error::new(attr.path.span(), "Unrecognized attribute"));
                }
            }

            if !input.peek(syn::Token!(impl)) {
                class = Some(Self::parse_class(input, docs, has_class_factory)?);
            } else {
                let item = input.parse::<syn::ItemImpl>()?;
                // TODO: ensure that class idents line up
                let interface = match item.trait_ {
                    Some((_, i, _)) => i,
                    None => {
                        return Err(syn::Error::new(
                            item.span(),
                            "Impl must be for an interface",
                        ))
                    }
                };
                let ms = item
                    .items
                    .into_iter()
                    .map(|i| match i {
                        syn::ImplItem::Method(m) => Ok(m),
                        _ => Err(syn::Error::new(
                            i.span(),
                            "only trait methods are allowed when implementing an interface",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;

                if methods.insert(interface.clone(), ms).is_some() {
                    return Err(syn::Error::new(interface.span(), "interface was redefined"));
                }
            }
        }
        let mut class = match class {
            Some(c) => {
                if c.requires_dispatch {
                    // This VTable ends up unimplemented because we expect all dispatches to go through our
                    // external IDispatch implementation.
                    let idispatch_path: syn::Path = syn::parse_quote! {::com::interfaces::IDispatch};

                    if let Some((orig, _)) = methods.get_key_value(&idispatch_path) {
                        return Err(syn::Error::new(orig.span(), "$IDispatch should not be manually implemented"));
                    }

                    let get_type_info_count: syn::ImplItemMethod = syn::parse_quote! {
                        fn GetTypeInfoCount(&self) {
                            unimplemented!()
                        }
                    };

                    let get_type_info: syn::ImplItemMethod = syn::parse_quote! {
                        fn GetTypeInfo(&self) {
                            unimplemented!()
                        }
                    };

                    let get_ids_of_names: syn::ImplItemMethod = syn::parse_quote! {
                        fn GetIDsOfNames(&self) {
                            unimplemented!()
                        }
                    };

                    let invoke: syn::ImplItemMethod = syn::parse_quote! {
                        fn Invoke(&self) {
                            unimplemented!()
                        }
                    };

                    methods.insert(idispatch_path, vec![
                        get_type_info_count,
                        get_type_info,
                        get_ids_of_names,                        
                        invoke
                    ]);
                }

                let mut interface_paths = c.interfaces_paths();
                for i in methods.keys() {
                    if !interface_paths.remove(i) {
                        return Err(syn::Error::new(
                            i.span(),
                            format!("impl for a non-declared interface: {:#?}", interface_paths),
                        ));
                    }
                }
                if let Some(i) = interface_paths.into_iter().next() {
                    return Err(syn::Error::new(i.span(), "impl for interface is missing"));
                }
                c
            }
            None => {
                return Err(syn::Error::new(input.span(), "no class was defined"));
            }
        };
        class.impl_debug = impl_debug;
        class.methods = methods;
        Ok(class)
    }
}

fn parse_derive_debug(attr: &syn::Attribute) -> syn::Result<()> {
    match attr.parse_meta() {
        Ok(syn::Meta::List(l))
            if matches!(l.nested.iter().next(), Some(syn::NestedMeta::Meta(syn::Meta::Path(p))) if p.is_ident("Debug"))
                && l.nested.len() == 1 =>
        {
            Ok(())
        }
        _ => Err(syn::Error::new(
            attr.tokens.span(),
            "Unrecognized derive attribute",
        )),
    }
}

mod keywords {
    syn::custom_keyword!(class);
    syn::custom_keyword!(factory);
}

#[derive(Debug)]
pub struct Interface {
    pub path: syn::Path,
    pub parent: Option<Box<Interface>>,

    // This is only ever set on the top-level Interface
    pub needs_dispatch_imp: bool,
}

impl Interface {
    /// Creates a METHODDATA array for the class's interface chain which contains IDispatch
    pub fn to_interface_data_ptr_tokens(&self) -> TokenStream {        
        quote! {
            let interface_data_ptr: *const ::com::interfaces::idispatch::InterfaceData = unsafe {
                static params: [::com::interfaces::idispatch::ParamData; 1] = [
                    ::com::interfaces::idispatch::ParamData {
                        name: std::ptr::null(),
                        var_type: ::com::TypeDescVarType::Bool,
                    }
                ];

                static dispatch_methods: [::com::interfaces::idispatch::MethodData; 1] = [
                    ::com::interfaces::idispatch::MethodData {
                        name: std::ptr::null(),
                        params: &params as *const _,
                        dispatch_id: ::com::interfaces::idispatch::DispatchId(402),
                        method_id: 41,
                        calling_convention: ::com::interfaces::idispatch::CallingConvention::StdCall,
                        params_count: 1,
                        flags: 4,
                        return_type: ::com::TypeDescVarType::Empty,
                    }
                ];

                static interface: ::com::interfaces::idispatch::InterfaceData = ::com::interfaces::idispatch::InterfaceData {
                    methods: &dispatch_methods as *const _,
                    method_count: 1,
                };

                &interface as *const _
            };
        }
    }

    /// Creates an intialized VTable for the interface
    pub fn to_initialized_vtable_tokens(&self, class: &Class, offset: usize) -> TokenStream {
        let class_name = &class.name;
        let vtable_ident = self.vtable_ident();
        let vtable_type = self.to_vtable_type_tokens();
        let parent = match self.parent.as_ref() {
            Some(p) => p.to_initialized_vtable_tokens(class, offset),
            None => Self::iunknown_tokens(class, offset),
        };
        let fields = class.methods.get(&self.path).unwrap().iter().map(|m| {
            let name = &m.sig.ident;
            let params = m.sig.inputs.iter().filter_map(|p| {
                match p {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(p) => Some(p),
                }
            });
            let args = params.clone().map(|p| &p.pat);
            let translation = params.clone().map(|p| {
                let pat = &p.pat;
                let typ = &p.ty;
                quote! {
                    let #pat = <#typ as ::com::AbiTransferable>::from_abi(#pat);
                }
            });
            let params = params.map(|p| {
                let pat = &p.pat;
                let typ = &p.ty;
                quote! {
                    #pat: <#typ as ::com::AbiTransferable>::Abi
                }
            });
            let ret = &m.sig.output;
            let method = quote! {
                #[allow(non_snake_case)]
                unsafe extern "system" fn #name(this: ::core::ptr::NonNull<::core::ptr::NonNull<#vtable_ident>>, #(#params),*) #ret {
                    let this = this.as_ptr().sub(#offset);
                    let this = ::core::mem::ManuallyDrop::new(::com::production::ClassAllocation::from_raw(this as *mut _ as *mut #class_name));
                    #(#translation)*
                    #class_name::#name(&this, #(#args),*)
                }
            };
            let field_name = Ident::new(&crate::utils::snake_to_camel(&name.to_string()), proc_macro2::Span::call_site());
            quote! {
                #field_name: {
                    #method
                    #name
                }
            }
        });
        quote! {
            {
                #vtable_type
                #vtable_ident {
                    parent: #parent,
                    #(#fields),*
                }
            }
        }
    }

    fn to_vtable_type_tokens(&self) -> TokenStream {
        let name = &self.path;
        let vtable_ident = self.vtable_ident();
        quote! {
            type #vtable_ident = <#name as ::com::Interface>::VTable;
        }
    }

    fn vtable_ident(&self) -> proc_macro2::Ident {
        let name = &self.path;
        quote::format_ident!("{}VTable", name.segments.last().unwrap().ident)
    }

    fn iunknown_tokens(class: &Class, offset: usize) -> TokenStream {
        let iunknown = super::iunknown_impl::IUnknownAbi::new(class.name.clone(), offset);
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens();
        let query_interface = iunknown.to_query_interface_tokens();
        quote! {
            {
                type IUknownVTable = <::com::interfaces::IUnknown as ::com::Interface>::VTable;
                #add_ref
                #release
                #query_interface
                IUknownVTable {
                    AddRef,
                    Release,
                    QueryInterface,
                }
            }
        }
    }
}
