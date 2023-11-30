extern crate proc_macro;

mod derive;

use quote::quote;

#[proc_macro_derive(EmbedAsAttrs, attributes(escape_value, prefix, suffix, rename))]
pub fn reflect_attrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    derive::expand_embed_as_attrs(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn html(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    //
    quote! {{
        extern crate lfml;

        lfml::Escaped("foo")
    }}
    .into()
}
