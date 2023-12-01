pub use lfml_escape::{escape_string, escape_to_string};
pub use lfml_macros::{html, EmbedAsAttrs};

pub trait EmbedAsAttrs {
    fn raw(&self) -> String;
}

pub struct Escaped<T>(pub T);

impl<T: std::fmt::Display> Escaped<T> {
    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

#[cfg(test)]
mod tests {}
