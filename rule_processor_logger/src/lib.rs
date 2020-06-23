//! This crate is supposed to support [jsonschema-equivalent](https://crates.io/crates/jsonschema-equivalent) by exporting
//! `log_processing` procedural macro.
//!
//! Plese don't use this outside of `jsonschema-equivalent` context, and if you do that's on your own risk.
//! Please refer to [`jsonschema-equivalent`](https://docs.rs/jsonschema-equivalent) docs for more informaton.
#![warn(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::integer_arithmetic,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::result_map_unwrap_or_else,
    clippy::result_unwrap_used,
    clippy::trivially_copy_pass_by_ref,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat};

/// Procedural macro that allows to wrap the annotated method into a logging structure.
///
/// The following code
/// ```
/// # use jsonschema_equivalent_rule_processor_logger::log_processing;
/// #[log_processing]
/// fn foo(schema: &mut serde_json::Value) -> bool {
///     false
/// }
/// ```
///
/// will result, after procedural macro expansion, roughtly equivalent to the following:
/// ```rust
/// fn foo(schema: &mut serde_json::Value) -> bool {
///     // NOTE: The details might deffer in different versions. This is presented as example only.
///     let original_schema = schema.clone();
///     let start = std::time::Instant::now();
///
///     # let original_function_block = true;
///     let result = original_function_block;
///
///     {log::info!("{}", serde_json::json!({
///         "method": "foo",
///         "elapsed_time_s": (std::time::Instant::now() - start),
///         "input_schema": original_schema,
///         "output_schema": schema,
///         "is_schema_updated": result
///     }));}
///     result
/// }
/// ```
///
/// **NOTE**: You can also decide to have some feature gating for the logging logic
/// ```
/// # use jsonschema_equivalent_rule_processor_logger::log_processing;
/// #[log_processing(cfg(feature = "my-feature"))]
/// fn bar(schema: &mut serde_json::Value) -> bool {
///     false
/// }
/// ```
/// will be expanded to something like
/// ```rust
/// fn bar(schema: &mut serde_json::Value) -> bool {
///     // NOTE: The details might deffer in different versions. This is presented as example only.
///     #[cfg(feature = "my-feature")]
///     let original_schema = schema.clone();
///     #[cfg(feature = "my-feature")]
///     let start = std::time::Instant::now();
///
///     # let original_function_block = true;
///     let result = original_function_block;
///
///     #[cfg(feature = "my-feature")]
///     {log::info!("{}", serde_json::json!({
///         "method": "foo",
///         "elapsed_time_s": (std::time::Instant::now() - start),
///         "input_schema": original_schema,
///         "output_schema": schema,
///         "is_schema_updated": result
///     }));}
///     result
/// }
/// ```
///

#[proc_macro_attribute]
pub fn log_processing(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Break the function down into its parts
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);

    let maybe_input_param_name = if let FnArg::Typed(pat_type) = &sig.inputs[0] {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            Some(&pat_ident.ident)
        } else {
            None
        }
    } else {
        None
    };
    let input_param_name = maybe_input_param_name.expect("Expected parameter name. If the signature is `fn(T, ...) -> bool` then we'll be able to extract it");
    let method_name = sig.ident.to_string();

    let maybe_gating_attribute = if attr.is_empty() {
        quote! {}
    } else {
        let attr2: proc_macro2::TokenStream = attr.into();
        quote! {
            #[#attr2]
        }
    };
    let output = quote! {
        #(#attrs)*
        #vis #sig {
            #maybe_gating_attribute
            let input_schema: serde_json::Value = #input_param_name.clone();
            #maybe_gating_attribute
            let start = std::time::Instant::now();

            let is_schema_updated = #block;

            #maybe_gating_attribute
            {
                log::info!("{}", serde_json::json!({
                    "method": #method_name,
                    "elapsed_time_s": format!("{:.9}", (std::time::Instant::now() - start).as_secs_f64()),
                    "input_schema": input_schema,
                    "output_schema": #input_param_name,
                    "is_schema_updated": is_schema_updated
                }));
            }

            is_schema_updated
        }
    };

    output.into()
}
