use quote::quote;

pub fn generate_markup_expr(
    _input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    Ok(quote! {})
}
