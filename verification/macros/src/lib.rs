#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, DeriveInput, Expr, Lit, MetaNameValue, Token};

/// Generate the zero-runtime-cost `ProofState` metadata implementation used
/// by generated Kani and Verus proof families.
#[proc_macro_derive(ProofState)]
pub fn derive_proof_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::tokio_verification::ProofState for #ident #ty_generics #where_clause {
            const PROOF_TYPE_NAME: &'static str = ::core::stringify!(#ident);
        }
    }
    .into()
}

/// Attach validated proof-family metadata to a verification item.
///
/// Example:
///
/// ```ignore
/// #[proof_family(id = "task-state", backend = "kani", assurance = "abstract-model")]
/// fn proof() { ... }
/// ```
///
/// The item is emitted byte-for-byte unchanged after metadata validation.
/// Proof metadata is tracked by the external registry, so this attribute is
/// representation-neutral even when used on methods.
#[proc_macro_attribute]
pub fn proof_family(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
    let args = match parser.parse(attr) {
        Ok(args) => args,
        Err(error) => return error.into_compile_error().into(),
    };

    let mut id = None;
    let mut backend = None;
    let mut assurance = None;

    for arg in args {
        let Some(name) = arg.path.get_ident().map(ToString::to_string) else {
            return syn::Error::new_spanned(arg.path, "expected a simple metadata key")
                .into_compile_error()
                .into();
        };
        let Expr::Lit(expr_lit) = arg.value else {
            return syn::Error::new_spanned(arg.value, "expected a string literal")
                .into_compile_error()
                .into();
        };
        let Lit::Str(value) = expr_lit.lit else {
            return syn::Error::new_spanned(expr_lit, "expected a string literal")
                .into_compile_error()
                .into();
        };

        match name.as_str() {
            "id" => id = Some(value),
            "backend" => backend = Some(value),
            "assurance" => assurance = Some(value),
            _ => {
                return syn::Error::new_spanned(
                    arg.path,
                    "supported keys are id, backend, and assurance",
                )
                .into_compile_error()
                .into()
            }
        }
    }

    let Some(id) = id else {
        return syn::Error::new(proc_macro2::Span::call_site(), "missing id = \"...\"")
            .into_compile_error()
            .into();
    };
    let Some(backend) = backend else {
        return syn::Error::new(proc_macro2::Span::call_site(), "missing backend = \"...\"")
            .into_compile_error()
            .into();
    };
    let assurance = assurance.unwrap_or_else(|| syn::LitStr::new("unspecified", id.span()));

    match backend.value().as_str() {
        "kani" | "verus" | "loom" | "test" => {}
        other => {
            return syn::Error::new_spanned(
                &backend,
                format!("unsupported proof backend `{other}`"),
            )
            .into_compile_error()
            .into()
        }
    }

    if id.value().trim().is_empty() || assurance.value().trim().is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "proof family id and assurance must be non-empty",
        )
        .into_compile_error()
        .into();
    }

    item
}
