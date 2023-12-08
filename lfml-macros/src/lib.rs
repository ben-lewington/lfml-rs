extern crate proc_macro;

mod derive;
mod html;

#[proc_macro_derive(MarkupAttrs, attributes(prefix, suffix, rename, escape_value, tags))]
pub fn reflect_attrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    derive::expand_embed_as_attrs(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    html::generate_markup_expr(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
