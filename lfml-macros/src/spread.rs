mod generate;
mod parse;
mod syntax;

use crate::spread::syntax::SpreadInput;

const DATA_PREFIX: &str = "data";

pub fn generate_spread_impl(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut output = proc_macro2::TokenStream::new();

    let input = SpreadInput::parse(input.clone())?;

    generate::generate_spread_impl(input, &mut output)?;

    Ok(output)
}
