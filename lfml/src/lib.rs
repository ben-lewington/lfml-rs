pub use lfml_escape::{escape_string, escape_to_string};
pub use lfml_macros::{html, EmbedAsAttrs};

pub trait EmbedAsAttrs {
    fn raw(&self) -> String;
}

pub struct Escaped<T>(pub T);

#[cfg(test)]
mod tests {}
