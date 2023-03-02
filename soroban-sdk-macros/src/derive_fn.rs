use std::collections::HashMap;

use itertools::MultiUnzip;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use stellar_xdr::{
    ScEnvMetaEntry, ScEnvSpecialFn, ScEnvSpecialFnType, ScSpecEntry, ScSpecFunctionInputV0,
    ScSpecFunctionV0, ScSpecTypeDef, ScSymbol, StringM, VecM, WriteXdr,
};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Colon, Comma},
    Attribute, Error, FnArg, Ident, Pat, PatIdent, PatType, ReturnType, Type, TypePath,
};

use crate::{doc::docs_from_attrs, map_type::map_type};

#[allow(clippy::too_many_arguments)]
pub fn derive_fn(
    call: &TokenStream2,
    ty: &Type,
    ident: &Ident,
    attrs: &[Attribute],
    inputs: &Punctuated<FnArg, Comma>,
    output: &ReturnType,
    trait_ident: Option<&Ident>,
    client_ident: &str,
) -> Result<TokenStream2, TokenStream2> {
    // Collect errors as they are encountered and emit them at the end.
    let mut errors = Vec::<Error>::new();

    // Prepare the env input.
    let env_input = inputs.first().and_then(|a| match a {
        FnArg::Typed(pat_type) => {
            let ty = &*pat_type.ty;
            if let Type::Path(TypePath {
                path: syn::Path { segments, .. },
                ..
            }) = ty
            {
                if segments.last().map_or(false, |s| s.ident == "Env") {
                    Some(a)
                } else {
                    None
                }
            } else {
                None
            }
        }
        FnArg::Receiver(_) => None,
    });

    // Prepare the argument inputs.
    let (spec_args, wrap_args, wrap_calls): (Vec<_>, Vec<_>, Vec<_>) = inputs
        .iter()
        .skip(if env_input.is_some() { 1 } else { 0 })
        .enumerate()
        .map(|(i, a)| match a {
            FnArg::Typed(pat_type) => {
                let name = if let Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    pat_ident.ident.to_string()
                } else {
                    errors.push(Error::new(a.span(), "argument not supported"));
                    "".to_string()
                };
                let spec = match map_type(&pat_type.ty) {
                    Ok(type_) => {
                        let name = name.try_into().unwrap_or_else(|_| {
                            const MAX: u32 = 30;
                            errors.push(Error::new(ident.span(), format!("argument name too long, max length {} characters", MAX)));
                            StringM::<MAX>::default()
                        });
                        ScSpecFunctionInputV0{
                            doc: "".try_into().unwrap(),
                            name,
                            type_,
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        ScSpecFunctionInputV0{
                            doc: "".try_into().unwrap(),
                            name: "arg".try_into().unwrap(),
                            type_: ScSpecTypeDef::I32,
                        }
                    }
                };
                let ident = format_ident!("arg_{}", i);
                let arg = FnArg::Typed(PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(PatIdent {
                        ident: ident.clone(),
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        subpat: None,
                    })),
                    colon_token: Colon::default(),
                    ty: Box::new(Type::Verbatim(quote! { soroban_sdk::RawVal })),
                });
                let call = quote! {
                    <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                        <_ as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::RawVal>>::try_from_val(
                            &env,
                            &#ident
                        )
                    )
                };
                (spec, arg, call)
            }
            FnArg::Receiver(_) => {
                errors.push(Error::new(a.span(), "self argument not supported"));
                (ScSpecFunctionInputV0{ doc: "".try_into().unwrap(), name: "".try_into().unwrap(), type_: ScSpecTypeDef::I32 } , a.clone(), quote! {})
            }
        })
        .multiunzip();

    // Prepare the output.
    let spec_result = match output {
        ReturnType::Type(_, ty) => vec![match map_type(ty) {
            Ok(spec) => spec,
            Err(e) => {
                errors.push(e);
                ScSpecTypeDef::I32
            }
        }],
        ReturnType::Default => vec![],
    };

    // Generated code parameters.
    let wrap_export_name = &format!("{}", ident);
    let hidden_mod_ident = format_ident!("__{}", ident);
    let deprecated_note = format!(
        "use `{}::new(&env, &contract_id).{}` instead",
        client_ident, &ident
    );
    let env_call = if env_input.is_some() {
        quote! { env.clone(), }
    } else {
        quote! {}
    };
    let slice_args: Vec<TokenStream2> = (0..wrap_args.len()).map(|n| quote! { args[#n] }).collect();
    let use_trait = if let Some(t) = trait_ident {
        quote! { use super::#t }
    } else {
        quote! {}
    };

    // Generated code spec.
    let spec_entry = ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
        doc: docs_from_attrs(attrs).try_into().unwrap(), // TODO: Truncate docs, or display friendly compile error.
        name: wrap_export_name.as_str().try_into().unwrap_or_else(|_| {
            const MAX: u32 = 10;
            errors.push(Error::new(
                ident.span(),
                format!(
                    "contract function name too long, max length {} characters",
                    MAX,
                ),
            ));
            ScSymbol::default()
        }),
        inputs: spec_args.try_into().unwrap_or_else(|_| {
            const MAX: u32 = 10;
            errors.push(Error::new(
                inputs.iter().nth(MAX as usize).span(),
                format!(
                    "contract function has too many parameters, max count {} parameters",
                    MAX,
                ),
            ));
            VecM::<_, MAX>::default()
        }),
        outputs: spec_result.try_into().unwrap(),
    });
    let spec_xdr = spec_entry.to_xdr().unwrap();
    let spec_xdr_lit = proc_macro2::Literal::byte_string(spec_xdr.as_slice());
    let spec_xdr_len = spec_xdr.len();
    let spec_ident = format_ident!("__SPEC_XDR_FN_{}", ident.to_string().to_uppercase());
    let spec_fn_ident = format_ident!("spec_xdr_{}", ident.to_string());

    // If errors have occurred, render them instead.
    if !errors.is_empty() {
        let compile_errors = errors.iter().map(Error::to_compile_error);
        return Err(quote! { #(#compile_errors)* });
    }

    // Generated code.
    Ok(quote! {
        #[doc(hidden)]
        #(#attrs)*
        #[cfg_attr(target_family = "wasm", link_section = "contractspecv0")]
        pub static #spec_ident: [u8; #spec_xdr_len] = #ty::#spec_fn_ident();

        impl #ty {
            #(#attrs)*
            pub const fn #spec_fn_ident() -> [u8; #spec_xdr_len] {
                *#spec_xdr_lit
            }
        }

        #[doc(hidden)]
        #(#attrs)*
        pub mod #hidden_mod_ident {
            use super::*;

            #[deprecated(note = #deprecated_note)]
            #[cfg_attr(target_family = "wasm", export_name = #wrap_export_name)]
            pub fn invoke_raw(env: soroban_sdk::Env, #(#wrap_args),*) -> soroban_sdk::RawVal {
                #use_trait;
                <_ as soroban_sdk::IntoVal<soroban_sdk::Env, soroban_sdk::RawVal>>::into_val(
                    #[allow(deprecated)]
                    &#call(
                        #env_call
                        #(#wrap_calls),*
                    ),
                    &env
                )
            }

            #[deprecated(note = #deprecated_note)]
            pub fn invoke_raw_slice(
                env: soroban_sdk::Env,
                args: &[soroban_sdk::RawVal],
            ) -> soroban_sdk::RawVal {
                #[allow(deprecated)]
                invoke_raw(env, #(#slice_args),*)
            }

            use super::*;
        }
    })
}

#[allow(clippy::too_many_lines)]
pub fn derive_contract_function_set<'a>(
    ty: &Type,
    methods: impl Iterator<Item = &'a syn::ImplItemMethod>,
    special_fns: &Vec<ScEnvSpecialFn>,
) -> TokenStream2 {
    let (idents, wrap_idents, attrs): (Vec<_>, Vec<_>, Vec<_>) = methods
        .map(|m| {
            let ident = format!("{}", m.sig.ident);
            let wrap_ident = format_ident!("__{}", m.sig.ident);
            let attrs = m
                .attrs
                .iter()
                // Don't propogate doc comments into the match statement below.
                .filter(|a| !a.path.is_ident("doc"))
                .collect::<Vec<_>>();
            (ident, wrap_ident, attrs)
        })
        .multiunzip();
    let (special_fn_types, special_fn_names): (Vec<_>, Vec<_>) = special_fns
        .iter()
        .map(|f| {
            (
                syn::parse_str::<syn::Expr>(
                    format!(
                        "soroban_sdk::xdr::ScEnvSpecialFnType::{}",
                        f.fn_type.to_string()
                    )
                    .as_str(),
                )
                .unwrap(),
                format!("{}", f.name.0.to_string_lossy().as_str()),
            )
        })
        .multiunzip();
    let ty_str = quote! {#ty}.to_string().to_uppercase();
    let special_fns_const_ident = format_ident!("__SPECIAL_FUNCTIONS_{}", ty_str);
    quote! {
        #[cfg(any(test, feature = "testutils"))]
        impl soroban_sdk::testutils::ContractFunctionSet for #ty {
            fn call(
                &self,
                func: &soroban_sdk::Symbol,
                env: soroban_sdk::Env,
                args: &[soroban_sdk::RawVal],
            ) -> Option<soroban_sdk::RawVal> {
                match ::core::convert::AsRef::<str>::as_ref(&func.to_str()) {
                    #(
                        #(#attrs)*
                        #idents => {
                            #[allow(deprecated)]
                            Some(#wrap_idents::invoke_raw_slice(env, args))
                        }
                    )*
                    _ => {
                        None
                    }
                }
            }

            fn special_functions(&self) -> &[(soroban_sdk::xdr::ScEnvSpecialFnType, &'static str)] {
                #special_fns_const_ident
            }
        }

        #[cfg(any(test, feature = "testutils"))]
        const #special_fns_const_ident: &[(soroban_sdk::xdr::ScEnvSpecialFnType, &'static str)] = &[
            #(
                (
                    #special_fn_types,
                    #special_fn_names
                ),
            )*
        ];
    }
}

pub fn derive_special_fn_spec(ty: &Type, special_fns: &Vec<ScEnvSpecialFn>) -> TokenStream2 {
    if special_fns.is_empty() {
        return quote!();
    }

    let env_meta = ScEnvMetaEntry::SpecialFunctions(special_fns.try_into().unwrap());
    let env_meta_xdr = env_meta.to_xdr().unwrap();
    let env_meta_xdr_len = env_meta_xdr.len();
    let env_meta_xdr_lit = proc_macro2::Literal::byte_string(env_meta_xdr.as_slice());
    let ty_str = quote! {#ty}.to_string().to_uppercase();
    let env_meta_ident = format_ident!("__ENV_META_SPECIAL_FNS_XDR_{}", ty_str);
    quote! {
        #[cfg_attr(target_family = "wasm", link_section = "contractenvmetav0")]
        pub static #env_meta_ident: [u8; #env_meta_xdr_len] = *#env_meta_xdr_lit;
    }
}

pub fn get_special_fns(
    custom_account_check_auth_fn: &Option<String>,
) -> HashMap<String, ScEnvSpecialFnType> {
    let mut res = HashMap::new();
    if let Some(f) = custom_account_check_auth_fn {
        res.insert(
            f.clone(),
            ScEnvSpecialFnType::ScEnvSpecialFnTypeCustomAccountCheckAuth,
        );
    }
    res
}
