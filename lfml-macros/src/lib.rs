extern crate proc_macro;

mod html;
mod spread;

#[proc_macro_derive(Spread, attributes(prefix, suffix, rename, escape_value, tags))]
pub fn spread(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    spread::generate_spread_impl(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    html::generate_markup_expr(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
