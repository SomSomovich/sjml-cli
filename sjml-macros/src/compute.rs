use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse2, Expr, FnArg, GenericArgument, ItemFn, Lit, Meta, PathArguments, ReturnType, Safety,
    Token, Type,
};

pub(crate) fn expand_compute(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let name = match callback_name(attribute) {
        Ok(name) => name,
        Err(error) => return error.to_compile_error(),
    };
    let function = match parse2::<ItemFn>(item) {
        Ok(function) => function,
        Err(error) => return error.to_compile_error(),
    };
    if let Some(reason) = validate_function(&function) {
        return syn::Error::new(function.sig.ident.span(), reason).to_compile_error();
    }
    let input_type = match function.sig.inputs.first() {
        Some(FnArg::Typed(argument)) => argument.ty.as_ref(),
        _ => unreachable!("validate_function checked the compute callback argument"),
    };
    let output_type = match result_value_type(&function.sig.output) {
        Some(output_type) => output_type,
        None => {
            return syn::Error::new(
                function.sig.ident.span(),
                "#[compute] function must return SjmlResult<T>",
            )
            .to_compile_error();
        }
    };
    let handler = &function.sig.ident;
    let visibility = &function.vis;
    let wrapper = format_ident!("__sjml_v1_compute_{}", handler);
    let spawn_wrapper = format_ident!("{}_compute", handler);
    let export_name = format!("sjml_v1_compute_{name}");
    quote! {
        #function

        #[doc(hidden)]
        #[cfg(target_arch = "wasm32")]
        #[export_name = #export_name]
        /// # Safety
        ///
        /// Non-empty input must be a live allocation from `sjml_v1_alloc`
        /// in this guest instance. The caller must initialize the exact span
        /// before this call and keep it live and unmodified until return.
        pub unsafe extern "C" fn #wrapper(input_ptr: u32, input_len: u32) -> u64 {
            // SAFETY: the isolated worker host guarantees this guest allocation.
            unsafe {
                ::sjml::__private::dispatch_payload::<#input_type, #output_type, _>(
                    input_ptr,
                    input_len,
                    #handler,
                )
            }
        }

        #[doc = "Creates a typed future for the isolated `#[compute]` worker callback."]
        #visibility fn #spawn_wrapper(
            input: #input_type,
        ) -> ::sjml::SjmlResult<::sjml::task::ComputeFuture<#output_type>> {
            ::sjml::task::spawn_compute::<#input_type, #output_type>(#name, &input)
        }
    }
}

fn callback_name(attribute: TokenStream2) -> syn::Result<String> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let arguments = parser.parse2(attribute)?;
    if arguments.len() != 1 {
        return Err(syn::Error::new(
            arguments.span(),
            "compute attribute requires exactly one name = \"...\" argument",
        ));
    }
    let argument = arguments.first().expect("one argument was checked");
    let name = match argument {
        Meta::NameValue(value) if value.path.is_ident("name") => match &value.value {
            Expr::Lit(expression) => match &expression.lit {
                Lit::Str(name) => name.value(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        &value.value,
                        "name must be a string",
                    ))
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    &value.value,
                    "name must be a string",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                argument,
                "compute attribute accepts only name = \"...\"",
            ))
        }
    };
    if !valid_callback_identifier(&name) {
        return Err(syn::Error::new_spanned(
            argument,
            "compute name must be a lowercase ASCII identifier: start with a-z or 0-9, then use only a-z, 0-9, '.', '_' or '-' (maximum 128 bytes)",
        ));
    }
    Ok(name)
}

fn valid_callback_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && matches!(value.as_bytes()[0], b'a'..=b'z' | b'0'..=b'9')
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn validate_function(function: &ItemFn) -> Option<String> {
    if function.sig.asyncness.is_some() {
        return Some("compute callbacks must be synchronous functions".to_owned());
    }
    if matches!(function.sig.safety, Safety::Unsafe(_)) {
        return Some("compute callbacks must be safe Rust functions".to_owned());
    }
    if !function.sig.generics.params.is_empty() {
        return Some("compute callbacks cannot be generic functions".to_owned());
    }
    if function.sig.variadic.is_some() {
        return Some("compute callbacks cannot be variadic".to_owned());
    }
    if function.sig.inputs.len() != 1 {
        return Some("compute callback must accept exactly one typed payload argument".to_owned());
    }
    if !matches!(function.sig.inputs.first(), Some(FnArg::Typed(_))) {
        return Some("compute callbacks must be free functions, not methods".to_owned());
    }
    None
}

fn result_value_type(output: &ReturnType) -> Option<&Type> {
    let ReturnType::Type(_, ty) = output else {
        return None;
    };
    let Type::Path(path) = ty.as_ref() else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "SjmlResult" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    match arguments.args.first()? {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::expand_compute;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn diagnostic(tokens: TokenStream, expected: &str) {
        let rendered = tokens.to_string();
        assert!(
            rendered.contains("compile_error"),
            "expected compile_error!: {rendered}"
        );
        assert!(
            rendered.contains(expected),
            "expected {expected:?}: {rendered}"
        );
    }

    #[test]
    fn compute_macro_generates_typed_worker_and_export_wrappers() {
        let generated = expand_compute(
            quote!(name = "demo.digest"),
            quote! {
                pub fn digest(input: DigestRequest) -> SjmlResult<DigestResponse> { Ok(input.into()) }
            },
        )
        .to_string();
        assert!(generated.contains("sjml_v1_compute_demo.digest"));
        assert!(generated.contains("dispatch_payload"));
        assert!(generated.contains("digest_compute"));
        assert!(generated.contains("spawn_compute"));
    }

    #[test]
    fn compute_macro_rejects_async_handlers_and_invalid_names() {
        diagnostic(
            expand_compute(
                quote!(name = "demo.digest"),
                quote! { async fn digest(input: u32) -> SjmlResult<u32> { Ok(input) } },
            ),
            "must be synchronous",
        );
        diagnostic(
            expand_compute(
                quote!(name = "Bad name"),
                quote! { fn digest(input: u32) -> SjmlResult<u32> { Ok(input) } },
            ),
            "lowercase ASCII identifier",
        );
    }

    #[test]
    fn compute_macro_requires_a_typed_callback_result() {
        diagnostic(
            expand_compute(
                quote!(name = "demo.digest"),
                quote! { fn digest() -> SjmlResult<u32> { Ok(1) } },
            ),
            "exactly one typed payload argument",
        );
        diagnostic(
            expand_compute(
                quote!(name = "demo.digest"),
                quote! { fn digest(input: u32) -> u32 { input } },
            ),
            "must return SjmlResult<T>",
        );
    }
}
