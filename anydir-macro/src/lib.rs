use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Embed a directory at compile time
#[proc_macro]
pub fn embed_dir(input: TokenStream) -> TokenStream {
    // Parse the input as a string literal
    let input = parse_macro_input!(input as LitStr);
    let dir_path = input.value();

    // Generate a unique identifier for the static variable
    let var_name = sanitize_identifier(&dir_path);

    let var_ident = syn::Ident::new(&var_name, proc_macro2::Span::call_site());

    // Generate the code
    let expanded = quote! {
        {
            use include_dir::include_dir;
            static #var_ident: include_dir::Dir = include_dir!(#dir_path);
            &#var_ident
        }
    };

    TokenStream::from(expanded)
}

fn sanitize_identifier(s: &str) -> String {
    let mut result = String::from("DIR_");
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_uppercase());
        } else {
            result.push('_');
        }
    }
    result
}
