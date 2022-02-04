extern crate proc_macro;

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

const SEVERITIES: &[&str] = &[
    "invalid_user_input",
    "invalid_client_input",
    "warning",
    "bug",
    "critical",
];

fn single_path(path: &syn::Path) -> Option<syn::Ident> {
    if path.segments.len() == 1 {
        Some(path.segments[0].ident.clone())
    } else {
        None
    }
}

fn parse_string_lit(lit: &syn::Lit) -> Result<String, syn::Error> {
    let got = match lit {
        syn::Lit::Str(predicate_str) => return Ok(predicate_str.value()),
        syn::Lit::ByteStr(_) => "ByteStr",
        syn::Lit::Byte(_) => "Byte",
        syn::Lit::Char(_) => "Char",
        syn::Lit::Int(_) => "Int",
        syn::Lit::Float(_) => "Float",
        syn::Lit::Bool(_) => "Bool",
        syn::Lit::Verbatim(_) => "Verbatim",
    };
    Err(syn::Error::new(
        lit.span(),
        format!("Invalid attribute, expected string but got {}", got),
    ))
}

fn generate(item_struct: syn::ItemEnum) -> Result<TokenStream, syn::Error> {
    let struct_name = item_struct.ident;

    let mut error_code_match_arms = Vec::with_capacity(item_struct.variants.len());
    let mut severity_match_arms = Vec::with_capacity(item_struct.variants.len());
    let mut tags_match_arms = Vec::with_capacity(item_struct.variants.len());

    for variant in item_struct.variants.into_iter() {
        let variant_span = variant.span();
        let error_code = variant.ident.to_string().to_case(Case::Kebab);
        let variant_ident = variant.ident;
        let mut tags = BTreeMap::new();
        let mut severity = None;
        for attr in &variant.attrs {
            if let syn::Meta::List(list) = attr.parse_meta()? {
                let path = single_path(&list.path).ok_or_else(|| {
                    syn::Error::new(
                        list.path.span(),
                        "Invalid attribute, must be single path element",
                    )
                })?;
                let path_str = path.to_string();
                if !SEVERITIES.iter().any(|&severity| path_str == severity) {
                    continue;
                }
                severity = Some(path);
                for nested_attr in list.nested.into_iter() {
                    match nested_attr {
                        syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                            let field = single_path(&name_value.path).ok_or_else(|| {
                                syn::Error::new(attr.span(), "Invalid attribute, invalid name")
                            })?;
                            let field_str = field.to_string();
                            if tags.contains_key(&field_str) {
                                return Err(syn::Error::new(
                                    attr.span(),
                                    format!("Duplicate tag: {}", field_str),
                                ));
                            }
                            tags.insert(field_str, parse_string_lit(&name_value.lit)?);
                        }
                        _ => {
                            return Err(syn::Error::new(
                                nested_attr.span(),
                                "Invalid literal, must provide values like this: a=<value>",
                            ))
                        }
                    }
                }
            }
        }
        let severity_pascal = severity
            .as_ref()
            .ok_or_else(|| {
                syn::Error::new(
                    variant_span,
                    "Missing severity, must be one of invalid_user_input, invalid_client_input, \
                     warning, bug, critical.",
                )
            })?
            .to_string()
            .to_case(Case::Pascal);
        let severity = Ident::new(&severity_pascal, severity.span());
        let variant_match_part = match &variant.fields {
            syn::Fields::Named(_) => quote! { {..} },
            syn::Fields::Unnamed(_) => quote! { (..) },
            syn::Fields::Unit => quote! {},
        };
        error_code_match_arms.push(quote! {
            #struct_name::#variant_ident #variant_match_part => {
                ::std::borrow::Cow::Borrowed(#error_code)
            }
        });
        severity_match_arms.push(quote! {
            #struct_name::#variant_ident #variant_match_part => {
                ::bitcoinsuite_error::ErrorSeverity::#severity
            }
        });
        let tag_keys = tags.keys();
        let tag_values = tags.values();
        tags_match_arms.push(quote! {
            #struct_name::#variant_ident #variant_match_part => ::std::borrow::Cow::Borrowed(&[
                #((
                    ::std::borrow::Cow::Borrowed(#tag_keys),
                    ::std::borrow::Cow::Borrowed(#tag_values),
                )),*
            ])
        });
    }

    let trait_impl = quote! {
        impl ::bitcoinsuite_error::ErrorMeta for #struct_name {
            fn severity(&self) -> ::bitcoinsuite_error::ErrorSeverity {
                match self {
                    #(#severity_match_arms),*
                }
            }
            fn error_code(&self) -> ::std::borrow::Cow<'static, str> {
                match self {
                    #(#error_code_match_arms),*
                }
            }
            fn tags(&self) -> ::std::borrow::Cow<'static, [
                (::std::borrow::Cow<'static, str>, ::std::borrow::Cow<'static, str>)
            ]> {
                match self {
                    #(#tags_match_arms),*
                }
            }
        }
    };
    Ok(trait_impl)
}

#[proc_macro_derive(
    ErrorMeta,
    attributes(invalid_user_input, invalid_client_input, warning, bug, critical)
)]
pub fn error_meta_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_enum = syn::parse_macro_input!(item as syn::ItemEnum);

    let result = match generate(item_enum) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    result.into()
}
