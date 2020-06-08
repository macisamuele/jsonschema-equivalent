use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat};

#[proc_macro_attribute]
pub fn log_processing(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let input_param_name = maybe_input_param_name.expect("Expected parameter name. If the signature is `fn(T) -> T` then we'll be able to extract it");
    let method_name = sig.ident.to_string();

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            cfg_if::cfg_if! {
                if #[cfg(feature = "logging")] {
                    let input_schema: serde_json::Value = #input_param_name.clone();
                    let start = std::time::Instant::now();
                }
            }

            let result = #block;

            cfg_if::cfg_if! {
                if #[cfg(feature = "logging")] {
                    let end = std::time::Instant::now();
                    let log_final_schema: serde_json::Value = result.clone();
                    log::info!("{}", serde_json::json!({
                        "method": #method_name,
                        "elapsed_time_s": (end - start).as_secs_f64(),
                        "input_schema": input_schema,
                        "output_schema": result,
                        "is_schema_updated": result != &input_schema,
                    }));
                }
            }

            result
        }
    };

    // Convert the output from a `proc_macro2::TokenStream` to a `proc_macro::TokenStream`
    TokenStream::from(output)
}
