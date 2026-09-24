//! Procedural macros for compact verification code.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::{parse_macro_input, DeriveInput, Fields, ItemFn, LitInt, LitStr};

#[derive(Default)]
struct KaniProofArgs {
    id: Option<LitStr>,
    unwind: Option<LitInt>,
}

impl KaniProofArgs {
    fn parse(meta: ParseNestedMeta<'_>, out: &mut Self) -> syn::Result<()> {
        if meta.path.is_ident("id") {
            out.id = Some(meta.value()?.parse()?);
            return Ok(());
        }
        if meta.path.is_ident("unwind") {
            out.unwind = Some(meta.value()?.parse()?);
            return Ok(());
        }
        Err(meta.error("supported keys are id and unwind"))
    }
}

/// Convert a zero-argument function into a Kani proof harness.
#[proc_macro_attribute]
pub fn kani_proof(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut parsed = KaniProofArgs::default();
    let parser = syn::meta::parser(|meta| KaniProofArgs::parse(meta, &mut parsed));
    parse_macro_input!(args with parser);

    let function = parse_macro_input!(input as ItemFn);
    if !function.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &function.sig.inputs,
            "generated Kani proof harnesses must take no arguments",
        )
        .to_compile_error()
        .into();
    }

    let id = match parsed.id {
        Some(id) => id,
        None => {
            return syn::Error::new_spanned(
                &function.sig.ident,
                "kani_proof requires id = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };
    let unwind = parsed
        .unwind
        .unwrap_or_else(|| LitInt::new("16", function.sig.ident.span()));

    let attrs = &function.attrs;
    let vis = &function.vis;
    let sig = &function.sig;
    let block = &function.block;

    quote! {
        #[cfg(kani)]
        #(#attrs)*
        #[kani::proof]
        #[kani::unwind(#unwind)]
        #vis #sig {
            const _: &str = #id;
            #block
        }
    }
    .into()
}

fn proof_family_name(attrs: &[syn::Attribute]) -> syn::Result<LitStr> {
    for attr in attrs {
        if attr.path().is_ident("proof_family") {
            let mut name = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse()?);
                    Ok(())
                } else {
                    Err(meta.error("supported key is name"))
                }
            })?;
            if let Some(name) = name {
                return Ok(name);
            }
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "ProofFamily requires proof_family(name = \"...\")",
    ))
}

/// Generate a ProofFamily implementation carrying a stable family name.
#[proc_macro_derive(ProofFamily, attributes(proof_family))]
pub fn derive_proof_family(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = match proof_family_name(&input.attrs) {
        Ok(name) => name,
        Err(error) => return error.to_compile_error().into(),
    };
    let ident = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics crate::ProofFamily for #ident #ty_generics #where_clause {
            const PROOF_FAMILY: &'static str = #name;
        }
    }
    .into()
}

/// Delegate InvariantCheck to one field tagged invariant_delegate.
#[proc_macro_derive(DelegateInvariant, attributes(invariant_delegate))]
pub fn derive_delegate_invariant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => {
            return syn::Error::new_spanned(
                &ident,
                "DelegateInvariant supports structs only",
            )
            .to_compile_error()
            .into()
        }
    };

    let mut target = None;
    match fields {
        Fields::Named(fields) => {
            for field in fields.named {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("invariant_delegate"))
                {
                    if target.is_some() {
                        return syn::Error::new_spanned(
                            field,
                            "exactly one invariant_delegate field is required",
                        )
                        .to_compile_error()
                        .into();
                    }
                    target = field.ident;
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &ident,
                "DelegateInvariant requires named fields",
            )
            .to_compile_error()
            .into()
        }
    }

    let target = match target {
        Some(target) => target,
        None => {
            return syn::Error::new_spanned(
                &ident,
                "missing invariant_delegate field",
            )
            .to_compile_error()
            .into()
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics crate::InvariantCheck for #ident #ty_generics #where_clause {
            fn invariant_ok(&self) -> bool {
                crate::InvariantCheck::invariant_ok(&self.#target)
            }
        }
    }
    .into()
}
