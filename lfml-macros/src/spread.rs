mod generate;
mod parse;
mod syntax;

use self::syntax::SpreadInput;
const DATA_PREFIX: &'static str = "data";

pub fn generate_spread_impl(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut output = proc_macro2::TokenStream::new();

    println!("{input:?}");

    let input = SpreadInput::parse(input.clone())?;

    // println!("{input:?}");
    // generate::generate_spread_impl(input, &mut output)?;

    Ok(output)
}
