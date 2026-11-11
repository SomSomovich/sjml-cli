//! Expansion for non-cancellable asynchronous event callbacks.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse2, Expr, FnArg, GenericArgument, ItemFn, Lit, Meta, PathArguments, ReturnType, Safety,
    Token, Type,
};

/// Expands `#[event_async(name = "...")]` into the regular synchronous event ABI export.
/// The export queues the returned future on the module's server task runtime and returns
/// immediately; callback failures are reported through the baseline host logger.
pub fn expand_event_async(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let name = match callback_name(attribute) {
        Ok(name) => name,
        Err(error) => return error.to_compile_error(),
    };
    if name == "block.break" {
        return error(
            Span::call_site(),
            "block.break is cancellable and must use a synchronous #[event] handler",
        );
    }

    let function = match parse2::<ItemFn>(item) {
        Ok(function) => function,
        Err(error) => return error.to_compile_error(),
    };
    if let Some(reason) = validate_function(&function) {
        return error(function.sig.ident.span(), reason);
    }

    let input_type = match function.sig.inputs.first() {
        Some(FnArg::Typed(argument)) => argument.ty.as_ref(),
        _ => unreachable!("validate_function checked the callback argument count"),
    };
    let handler = &function.sig.ident;
    let wrapper = format_ident!("__sjml_v1_event_async_{}", handler);
    let export_name = format!("sjml_v1_event_{name}");
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
            // SAFETY: the caller guarantees the callback input allocation.
            unsafe {
                ::sjml::__private::dispatch_payload::<#input_type, (), _>(
                    input_ptr,
                    input_len,
                    |payload| {
                        let future = #handler(payload);
                        match ::sjml::task::spawn(async move {
                            if let Err(error) = future.await {
                                let message = ::std::format!(
                                    "asynchronous event callback '{}' failed: {}",
                                    #name,
                                    error,
                                );
                                let _ = ::sjml::host::log(::sjml::host::LogLevel::Error, &message);
                            }
                        }) {
                            Ok(_) => {}
                            Err(error) => {
                                let message = ::std::format!(
                                    "could not schedule asynchronous event callback '{}': {}",
                                    #name,
                                    error,
                                );
                                let _ = ::sjml::host::log(::sjml::host::LogLevel::Error, &message);
                            }
                        }
                        Ok(())
                    },
                )
            }
        }
    }
}

fn callback_name(attribute: TokenStream) -> syn::Result<String> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let arguments = parser.parse2(attribute)?;
    if arguments.len() != 1 {
        return Err(syn::Error::new(
            arguments.span(),
            "callback attribute requires exactly one name = \"...\" argument",
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
                "callback attribute accepts only name = \"...\"",
            ));
        }
    };
    if !valid_callback_identifier(&name) {
        return Err(syn::Error::new_spanned(
            argument,
            "callback name must be a lowercase ASCII identifier: start with a-z or 0-9, then use only a-z, 0-9, '.', '_' or '-' (maximum 128 bytes)",
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

fn validate_function(function: &ItemFn) -> Option<&'static str> {
    if function.sig.asyncness.is_none() {
        return Some("#[event_async] requires an async function");
    }
    if matches!(function.sig.safety, Safety::Unsafe(_)) {
        return Some("SJML event callbacks must be safe Rust functions");
    }
    if !function.sig.generics.params.is_empty() {
        return Some("SJML event callbacks cannot be generic functions");
    }
    if function.sig.variadic.is_some() {
        return Some("SJML event callbacks cannot be variadic");
    }
    if function.sig.inputs.len() != 1 {
        return Some("event callback must accept exactly one typed payload argument");
    }
    if !matches!(function.sig.inputs.first(), Some(FnArg::Typed(_))) {
        return Some("SJML event callbacks must be free functions, not methods");
    }
    if !returns_unit_result(&function.sig.output) {
        return Some("async event callback must return SjmlResult<()> when awaited");
    }
    None
}

fn returns_unit_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::Path(path) = ty.as_ref() else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "SjmlResult" {
        return false;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    if arguments.args.len() != 1 {
        return false;
    }
    matches!(
        arguments.args.first(),
        Some(GenericArgument::Type(Type::Tuple(tuple))) if tuple.elems.is_empty()
    )
}

fn error(span: Span, message: impl AsRef<str>) -> TokenStream {
    syn::Error::new(span, message.as_ref()).to_compile_error()
}

#[cfg(test)]
mod tests {
    use super::expand_event_async;
    use quote::quote;
    use syn::parse_quote;

    fn diagnostic(tokens: proc_macro2::TokenStream, expected: &str) {
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
    fn generates_a_sync_event_export_that_queues_and_logs_the_future() {
        let generated = expand_event_async(
            quote!(name = "player.joined"),
            quote! {
                async fn on_joined(event: PlayerJoined) -> SjmlResult<()> { Ok(()) }
            },
        )
        .to_string();

        assert!(generated.contains("sjml_v1_event_player.joined"));
        assert!(generated.contains("dispatch_payload"));
        assert!(generated.contains("task :: spawn"));
        assert!(generated.contains("future . await"));
        assert!(generated.contains("host :: log"));
    }

    #[test]
    fn requires_async_unit_result_and_one_typed_argument() {
        diagnostic(
            expand_event_async(
                quote!(name = "player.joined"),
                quote! { fn joined(_: ()) -> SjmlResult<()> { Ok(()) } },
            ),
            "requires an async function",
        );
        diagnostic(
            expand_event_async(
                quote!(name = "player.joined"),
                quote! { async fn joined(_: ()) -> SjmlResult<bool> { Ok(true) } },
            ),
            "must return SjmlResult<()> when awaited",
        );
        diagnostic(
            expand_event_async(
                quote!(name = "player.joined"),
                quote! { async fn joined() -> SjmlResult<()> { Ok(()) } },
            ),
            "exactly one typed payload argument",
        );
    }

    #[test]
    fn cancellable_block_break_handlers_are_forced_to_remain_synchronous() {
        diagnostic(
            expand_event_async(
                quote!(name = "block.break"),
                quote! { async fn broken(_: BlockBreak) -> SjmlResult<()> { Ok(()) } },
            ),
            "block.break is cancellable",
        );
    }

    #[test]
    fn rejects_invalid_names_unsafe_generic_and_variadic_handlers() {
        diagnostic(
            expand_event_async(
                quote!(name = "Bad.Name"),
                quote! { async unsafe fn joined(_: ()) -> SjmlResult<()> { Ok(()) } },
            ),
            "lowercase ASCII identifier",
        );
        diagnostic(
            expand_event_async(
                quote!(name = "player.joined"),
                quote! { async unsafe fn joined(_: ()) -> SjmlResult<()> { Ok(()) } },
            ),
            "must be safe Rust functions",
        );
        let generic: syn::ItemFn = parse_quote! {
            async fn joined<T>(_: T) -> SjmlResult<()> { Ok(()) }
        };
        diagnostic(
            expand_event_async(quote!(name = "player.joined"), quote!(#generic)),
            "cannot be generic functions",
        );
    }
}
