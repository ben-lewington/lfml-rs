mod types;

pub use crate::types::{
    attrs::MarkupAttrs,
    markup::{Escaped, Render},
};

pub use lfml_escape::{escape_string, escape_to_string};
pub use lfml_macros::{html, MarkupAttrs};

pub type Markup = Escaped<String>;
