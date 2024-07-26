use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, ItemStruct, Token};

#[proc_macro_derive(InterfaceImpl, attributes(interface, versions))]
pub fn derive_interface_impl(tokens: TokenStream) -> TokenStream {
    let s: ItemStruct = syn::parse(tokens).unwrap();
    let name = s.ident;
    let generics_with_bounds = s.generics;
    let generics_idents: Punctuated<&syn::Ident, Token![,]> = generics_with_bounds
        .params
        .iter()
        .map(|p| {
            let syn::GenericParam::Type(p) = p else {
                panic!("expected type generic");
            };
            &p.ident
        })
        .collect();

    let versions_attr = &s
        .attrs
        .iter()
        .find(|a| a.style == syn::AttrStyle::Outer && a.meta.path().is_ident("versions"))
        .expect("missing versions attribute")
        .meta;
    let versions = versions_attr
        .require_list()
        .expect("versions attribute should be a list of versions")
        .parse_args_with(Punctuated::<syn::LitInt, Token![,]>::parse_separated_nonempty)
        .expect("parsing versions failed");

    let interface_attr = s
        .attrs
        .into_iter()
        .find(|a| a.style == syn::AttrStyle::Outer && a.meta.path().is_ident("interface"))
        .expect("missing interface attribute")
        .meta;
    let interface = interface_attr
        .require_name_value()
        .expect("parsing interface failed");

    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(interface),
        ..
    }) = &interface.value
    else {
        panic!("expected string literal for interface");
    };
    let interface = interface.value();

    let interface_versions_and_variants: Vec<(syn::Ident, syn::Ident)> = versions
        .iter()
        .map(|v| {
            let num: u8 = v.base10_parse().unwrap();
            (format_ident!("{interface}{num:03}"), format_ident!("V{v}"))
        })
        .collect();

    let wrapped_variants = interface_versions_and_variants.iter().map(|(interface, variant)| {
        quote! { #variant(crate::VtableWrapper<vr::#interface, super::#name<#generics_idents>>) }
    });
    let interface_versions = interface_versions_and_variants
        .iter()
        .map(|(interface, _)| {
            quote! { vr::#interface::VERSION }
        });

    let get_vtable_arms = interface_versions_and_variants.iter().enumerate().map(|(index, (interface, variant))| {
        quote! {
            x if x == vr::#interface::VERSION => {
                Some(Box::new(|this: &std::sync::Arc<Self>| {
                    let vtable = this.vtables.0[#index]
                        .get_or_init(|| {
                            WrappedVtable::#variant(<Self as crate::Inherits<vr::#interface>>::new_wrapped(this))
                        });

                    let WrappedVtable::#variant(vtable) = vtable else {
                        unreachable!("vtable currently holding incorrect version {:?} (expected {})", vtable, stringify!(#variant))
                    };
                    &vtable.base as *const vr::#interface as *mut std::ffi::c_void
                }))
            }
            x if x
                .to_string_lossy()
                .strip_prefix("FnTable:")
                .is_some_and(|x| x == vr::#interface::VERSION.to_string_lossy()) =>
            {
                Some(Box::new(<Self as crate::Inherits<vr::#interface>>::init_fntable))
            }
        }
    });

    let debug_arms = interface_versions_and_variants.iter().map(|(_, variant)| {
        quote! { Self::#variant(_) => f.write_str(stringify!(#variant)) }
    });

    let mod_name = format_ident!("{}_gen", name.to_string().to_lowercase());
    let num_versions = versions.len();

    quote! {
        use #mod_name::Vtables;
        mod #mod_name {
            use crate::vr;
            use std::sync::OnceLock;
            use super::*;
            use std::ffi::CStr;

            pub struct Vtables #generics_with_bounds ([OnceLock<WrappedVtable <#generics_idents>>; #num_versions]);
            impl #generics_with_bounds Default for Vtables<#generics_idents> {
                fn default() -> Self {
                    Self(std::array::from_fn(|_| OnceLock::new()))
                }
            }
            enum WrappedVtable #generics_with_bounds {
                #(#wrapped_variants),*
            }

            unsafe impl #generics_with_bounds Sync for WrappedVtable<#generics_idents> {}
            unsafe impl #generics_with_bounds Send for WrappedVtable<#generics_idents> {}

            impl #generics_with_bounds std::fmt::Debug for WrappedVtable<#generics_idents> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    match self {
                        #(#debug_arms),*
                    }
                }
            }

            impl #generics_with_bounds crate::InterfaceImpl for super::#name<#generics_idents>{
                fn supported_versions() -> &'static [&'static CStr] {
                    &[
                        #(#interface_versions),*
                    ]
                }

                fn get_version(version: &std::ffi::CStr) -> Option<Box<dyn FnOnce(&std::sync::Arc<Self>) -> *mut std::ffi::c_void>> {
                    match version {
                        #(#get_vtable_arms)*
                        _ => None
                    }
                }
            }
        }
    }.into()
}
