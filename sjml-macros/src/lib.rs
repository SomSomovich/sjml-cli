use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse2, Data, DeriveInput, Expr, Fields, FnArg, GenericArgument, ItemFn, Lit, LitStr, Meta,
    PathArguments, ReturnType, Safety, Token, Type,
};

mod compute;
mod event_async;

#[proc_macro_attribute]
pub fn compute(attribute: TokenStream, item: TokenStream) -> TokenStream {
    compute::expand_compute(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn event_async(attribute: TokenStream, item: TokenStream) -> TokenStream {
    event_async::expand_event_async(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn mod_init(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_mod_init(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn mod_register(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_mod_register(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn mod_shutdown(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_mod_shutdown(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn event(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_event(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn command(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_command(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn service(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_service(attribute.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn recipe(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_recipe(attribute.into(), item.into()).into()
}

#[proc_macro_derive(StableEncode, attributes(sjml))]
pub fn derive_stable_encode(input: TokenStream) -> TokenStream {
    expand_stable_encode(input.into()).into()
}

#[proc_macro_derive(StableDecode, attributes(sjml))]
pub fn derive_stable_decode(input: TokenStream) -> TokenStream {
    expand_stable_decode(input.into()).into()
}

fn expand_mod_init(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_lifecycle(attribute, item, "mod_init", "sjml_v1_mod_init")
}

fn expand_mod_register(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_lifecycle(attribute, item, "mod_register", "sjml_v1_mod_register")
}

fn expand_mod_shutdown(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_lifecycle(attribute, item, "mod_shutdown", "sjml_v1_mod_shutdown")
}

fn expand_lifecycle(
    attribute: TokenStream2,
    item: TokenStream2,
    macro_name: &str,
    export_name: &str,
) -> TokenStream2 {
    if !attribute.is_empty() {
        return error(
            attribute.span(),
            format!("#[{macro_name}] does not accept arguments"),
        );
    }
    let function = match parse2::<ItemFn>(item) {
        Ok(function) => function,
        Err(error) => return error.to_compile_error(),
    };
    if let Some(reason) = validate_function(&function, 0) {
        return error(function.sig.ident.span(), reason);
    }
    if !returns_unit_result(&function.sig.output) {
        return error(
            function.sig.ident.span(),
            format!("#[{macro_name}] function must return SjmlResult<()>"),
        );
    }

    let name = &function.sig.ident;
    let wrapper = format_ident!("__sjml_v1_{macro_name}");
    quote! {
        #function

        #[doc(hidden)]
        #[cfg(target_arch = "wasm32")]
        #[export_name = #export_name]
        pub extern "C" fn #wrapper() -> i32 {
            match #name() {
                Ok(()) => 0,
                Err(error) => error.abi_code() as i32,
            }
        }
    }
}

fn expand_event(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_callback(attribute, item, "event")
}

fn expand_command(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_callback(attribute, item, "command")
}

fn expand_service(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_callback(attribute, item, "service")
}

fn expand_recipe(attribute: TokenStream2, item: TokenStream2) -> TokenStream2 {
    expand_callback(attribute, item, "recipe")
}

fn expand_callback(attribute: TokenStream2, item: TokenStream2, kind: &str) -> TokenStream2 {
    let name = match callback_name(attribute) {
        Ok(name) => name,
        Err(error) => return error.to_compile_error(),
    };
    let function = match parse2::<ItemFn>(item) {
        Ok(function) => function,
        Err(error) => return error.to_compile_error(),
    };
    if let Some(reason) = validate_function(&function, 1) {
        return error(function.sig.ident.span(), reason);
    }

    let input_type = match function.sig.inputs.first() {
        Some(FnArg::Typed(argument)) => argument.ty.as_ref(),
        _ => unreachable!("validate_function checked the callback argument count"),
    };
    let output_type = match result_value_type(&function.sig.output) {
        Some(output_type) => output_type,
        None => {
            return error(
                function.sig.ident.span(),
                format!("#[{kind}] function must return SjmlResult<T>"),
            );
        }
    };

    let handler = &function.sig.ident;
    let wrapper = format_ident!("__sjml_v1_{kind}_{}", handler);
    let export_name = format!("sjml_v1_{kind}_{name}");
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
                ::sjml::__private::dispatch_payload::<#input_type, #output_type, _>(
                    input_ptr,
                    input_len,
                    #handler,
                )
            }
        }
    }
}

fn callback_name(attribute: TokenStream2) -> syn::Result<String> {
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

fn validate_function(function: &ItemFn, expected_arguments: usize) -> Option<String> {
    if function.sig.asyncness.is_some() {
        return Some(
            "async callbacks are not available until the SJML async runtime is implemented"
                .to_owned(),
        );
    }
    if matches!(function.sig.safety, Safety::Unsafe(_)) {
        return Some("SJML callbacks must be safe Rust functions".to_owned());
    }
    if !function.sig.generics.params.is_empty() {
        return Some("SJML callbacks cannot be generic functions".to_owned());
    }
    if function.sig.variadic.is_some() {
        return Some("SJML callbacks cannot be variadic".to_owned());
    }
    if function.sig.inputs.len() != expected_arguments {
        return Some(if expected_arguments == 0 {
            "lifecycle functions must not accept arguments".to_owned()
        } else {
            "callback must accept exactly one typed payload argument".to_owned()
        });
    }
    if function
        .sig
        .inputs
        .iter()
        .any(|argument| !matches!(argument, FnArg::Typed(_)))
    {
        return Some("SJML callbacks must be free functions, not methods".to_owned());
    }
    None
}

fn returns_unit_result(output: &ReturnType) -> bool {
    matches!(result_value_type(output), Some(Type::Tuple(tuple)) if tuple.elems.is_empty())
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

fn expand_stable_encode(input: TokenStream2) -> TokenStream2 {
    expand_stable_derive(input, DeriveKind::Encode)
}

fn expand_stable_decode(input: TokenStream2) -> TokenStream2 {
    expand_stable_derive(input, DeriveKind::Decode)
}

#[derive(Clone, Copy)]
enum DeriveKind {
    Encode,
    Decode,
}

fn expand_stable_derive(input: TokenStream2, kind: DeriveKind) -> TokenStream2 {
    let derive = match parse2::<DeriveInput>(input) {
        Ok(derive) => derive,
        Err(error) => return error.to_compile_error(),
    };
    let metadata = match schema_metadata(&derive) {
        Ok(metadata) => metadata,
        Err(error) => return error.to_compile_error(),
    };
    let version = metadata.version;
    let schema_id = &metadata.schema_id;
    let fields = match &derive.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return error(
                    derive.ident.span(),
                    "StableEncode/StableDecode only support structs with named fields",
                );
            }
        },
        _ => {
            return error(
                derive.ident.span(),
                "StableEncode/StableDecode only support structs with named fields",
            );
        }
    };
    if derive.generics.lifetimes().next().is_some() {
        return error(
            derive.ident.span(),
            "stable decoding cannot derive borrowed struct fields; use owned fields",
        );
    }

    let type_name = &derive.ident;
    let mut generics = derive.generics.clone();
    {
        let where_clause = generics.make_where_clause();
        for field in fields {
            let field_type = &field.ty;
            let trait_path = match kind {
                DeriveKind::Encode => quote!(::sjml::StableEncode),
                DeriveKind::Decode => quote!(::sjml::StableDecode),
            };
            where_clause
                .predicates
                .push(syn::parse_quote!(#field_type: #trait_path));
        }
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    match kind {
        DeriveKind::Encode => {
            let field_names = fields.iter().filter_map(|field| field.ident.as_ref());
            quote! {
                impl #impl_generics ::sjml::StableEncode for #type_name #type_generics #where_clause {
                    fn encode(&self, encoder: &mut ::sjml::Encoder) -> ::sjml::SjmlResult<()> {
                        encoder.write_u32(#version)?;
                        #( ::sjml::StableEncode::encode(&self.#field_names, encoder)?; )*
                        Ok(())
                    }
                }
            }
        }
        DeriveKind::Decode => {
            let initializers = fields.iter().map(|field| {
                let name = field.ident.as_ref().expect("named fields were checked");
                let field_type = &field.ty;
                quote! {
                    #name: <#field_type as ::sjml::StableDecode>::decode(decoder)?
                }
            });
            quote! {
                impl #impl_generics ::sjml::StableDecode for #type_name #type_generics #where_clause {
                    fn decode(decoder: &mut ::sjml::Decoder<'_>) -> ::sjml::SjmlResult<Self> {
                        let schema_version = decoder.read_u32()?;
                        if schema_version != #version {
                            return Err(::sjml::SjmlError::new(
                                ::sjml::ErrorCode::IncompatibleAbiVersion,
                                "guest record schema version does not match the derived schema",
                            ));
                        }
                        Ok(Self { #( #initializers, )* })
                    }
                }

                impl #impl_generics ::sjml::storage::StableRecord for #type_name #type_generics #where_clause {
                    const SCHEMA_VERSION: u32 = #version;

                    fn schema_id() -> &'static str {
                        #schema_id
                    }
                }
            }
        }
    }
}

struct SchemaMetadata {
    version: u32,
    schema_id: TokenStream2,
}

fn schema_metadata(derive: &DeriveInput) -> syn::Result<SchemaMetadata> {
    let mut version = None;
    let mut schema_id = None;
    for attribute in derive
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("sjml"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("version") {
                if version.is_some() {
                    return Err(meta.error("schema version must only be declared once"));
                }
                let literal: syn::LitInt = meta.value()?.parse()?;
                let parsed = literal.base10_parse::<u32>()?;
                if parsed == 0 {
                    return Err(syn::Error::new_spanned(
                        literal,
                        "schema version must be greater than zero",
                    ));
                }
                version = Some(parsed);
                return Ok(());
            }

            if meta.path.is_ident("id") {
                if schema_id.is_some() {
                    return Err(meta.error("schema identity must only be declared once"));
                }
                let literal: LitStr = meta.value()?.parse()?;
                let value = literal.value();
                if value.is_empty()
                    || value.len() > 256
                    || value.chars().all(char::is_whitespace)
                    || value.chars().any(char::is_control)
                {
                    return Err(syn::Error::new_spanned(
                        literal,
                        "schema identity must contain 1..=256 non-blank, non-control UTF-8 bytes",
                    ));
                }
                schema_id = Some(literal);
                return Ok(());
            }

            Err(meta.error("only version = N and id = \"...\" are supported in #[sjml(...)]"))
        })?;
    }

    let version = version.ok_or_else(|| {
        syn::Error::new_spanned(
            &derive.ident,
            "stable derives require #[sjml(version = N)] with a positive integer",
        )
    })?;
    let schema_id = schema_id.map_or_else(
        || quote!(::core::any::type_name::<Self>()),
        |value| quote!(#value),
    );
    Ok(SchemaMetadata { version, schema_id })
}

fn error(span: Span, message: impl AsRef<str>) -> TokenStream2 {
    syn::Error::new(span, message.as_ref()).to_compile_error()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn mod_init_generates_the_stable_lifecycle_export_and_rejects_bad_signatures() {
        let generated = expand_mod_init(
            quote!(),
            quote! {
                fn initialize() -> SjmlResult<()> { Ok(()) }
            },
        );
        assert!(generated.to_string().contains("sjml_v1_mod_init"));
        assert!(generated.to_string().contains("initialize"));

        diagnostic(
            expand_mod_init(
                quote!(),
                quote! {
                    fn initialize(_value: u32) -> SjmlResult<()> { Ok(()) }
                },
            ),
            "must not accept arguments",
        );
    }

    #[test]
    fn mod_register_generates_a_separate_pre_registry_export() {
        let generated = expand_mod_register(
            quote!(),
            quote! { fn register() -> SjmlResult<()> { Ok(()) } },
        );
        assert!(generated.to_string().contains("sjml_v1_mod_register"));
        diagnostic(
            expand_mod_register(
                quote!(),
                quote! { fn register(_context: u32) -> SjmlResult<()> { Ok(()) } },
            ),
            "must not accept arguments",
        );
    }

    #[test]
    fn mod_shutdown_generates_its_lifecycle_export() {
        let generated = expand_mod_shutdown(
            quote!(),
            quote! {
                fn shutdown() -> SjmlResult<()> { Ok(()) }
            },
        );
        assert!(generated.to_string().contains("sjml_v1_mod_shutdown"));
        diagnostic(
            expand_mod_shutdown(
                quote!(unexpected),
                quote! {
                    fn shutdown() -> SjmlResult<()> { Ok(()) }
                },
            ),
            "does not accept arguments",
        );
    }

    #[test]
    fn event_macro_generates_a_typed_dispatch_and_rejects_async_handlers() {
        let generated = expand_event(
            quote!(name = "player.joined"),
            quote! {
                fn joined(value: PlayerJoined) -> SjmlResult<()> { Ok(()) }
            },
        );
        assert!(generated
            .to_string()
            .contains("sjml_v1_event_player.joined"));
        assert!(generated.to_string().contains("dispatch_payload"));
        diagnostic(
            expand_event(
                quote!(name = "player.joined"),
                quote! {
                    async fn joined(value: PlayerJoined) -> SjmlResult<()> { Ok(()) }
                },
            ),
            "async callbacks are not available until the SJML async runtime is implemented",
        );
    }

    #[test]
    fn command_macro_generates_the_named_command_export() {
        let generated = expand_command(
            quote!(name = "demo.echo"),
            quote! {
                fn echo(value: String) -> SjmlResult<String> { Ok(value) }
            },
        );
        assert!(generated.to_string().contains("sjml_v1_command_demo.echo"));
        assert!(generated.to_string().contains("dispatch_payload"));
        diagnostic(
            expand_command(
                quote!(name = "Bad ID"),
                quote! {
                    fn echo(value: String) -> SjmlResult<String> { Ok(value) }
                },
            ),
            "lowercase ASCII identifier",
        );
    }

    #[test]
    fn service_macro_generates_a_typed_service_export() {
        let generated = expand_service(
            quote!(name = "demo.player_count"),
            quote! {
                fn count(_: ()) -> SjmlResult<u32> { Ok(1) }
            },
        );
        assert!(generated
            .to_string()
            .contains("sjml_v1_service_demo.player_count"));
        assert!(generated.to_string().contains("dispatch_payload"));
        diagnostic(
            expand_service(
                quote!(name = "demo.player_count"),
                quote! {
                    fn count() -> SjmlResult<u32> { Ok(1) }
                },
            ),
            "must accept exactly one typed payload argument",
        );
    }

    #[test]
    fn recipe_macro_generates_a_typed_dispatch() {
        let generated = expand_recipe(
            quote!(name = "ruby_smelting"),
            quote! {
                fn ruby_smelting(value: RecipeMatchInput) -> SjmlResult<RecipeEvaluation> {
                    Ok(RecipeEvaluation::no_match())
                }
            },
        );
        assert!(generated
            .to_string()
            .contains("sjml_v1_recipe_ruby_smelting"));
        assert!(generated.to_string().contains("dispatch_payload"));
        diagnostic(
            expand_recipe(
                quote!(name = "bad/name"),
                quote! {
                    fn bad(_: RecipeMatchInput) -> SjmlResult<RecipeEvaluation> {
                        Ok(RecipeEvaluation::no_match())
                    }
                },
            ),
            "callback name must be a lowercase ASCII identifier",
        );
    }

    #[test]
    fn stable_derive_generates_versioned_field_encoding() {
        let generated = expand_stable_encode(quote! {
            #[sjml(version = 7, id = "demo.player_message")]
            struct PlayerMessage { player_id: u32, accepted: bool }
        });
        assert!(generated.to_string().contains("write_u32"));
        assert!(generated.to_string().contains("player_id"));
        assert!(generated.to_string().contains("accepted"));
    }

    #[test]
    fn stable_decode_exposes_the_schema_version_and_stable_identity() {
        let generated = expand_stable_decode(quote! {
            #[sjml(version = 7, id = "demo.player_message")]
            struct PlayerMessage { player_id: u32, accepted: bool }
        })
        .to_string();

        assert!(generated.contains("StableRecord"));
        assert!(generated.contains("SCHEMA_VERSION"));
        assert!(generated.contains("7u32"));
        assert!(generated.contains("schema_id"));
        assert!(generated.contains("demo.player_message"));
    }

    #[test]
    fn stable_derives_report_missing_versions_and_unsupported_enums() {
        diagnostic(
            expand_stable_encode(quote! { struct MissingVersion { value: u32 } }),
            "stable derives require",
        );
        diagnostic(
            expand_stable_decode(quote! {
                #[sjml(version = 1)]
                enum Unsupported { One }
            }),
            "only support structs with named fields",
        );
    }

    #[test]
    fn stable_derive_rejects_blank_schema_identities() {
        diagnostic(
            expand_stable_decode(quote! {
                #[sjml(version = 1, id = "   ")]
                struct BlankIdentity { value: u32 }
            }),
            "schema identity must contain",
        );
    }
}
