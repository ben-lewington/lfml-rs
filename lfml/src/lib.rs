mod integrations;
mod types;

pub use crate::types::{
    attrs::Spread,
    markup::{Escaped, Render},
};

pub use lfml_escape::{escape_string, escape_to_string};
pub use lfml_macros::{html, Spread};

pub type Markup = Escaped<String>;

pub const DOCTYPE: Escaped<&str> = Escaped(lfml_html5::DOCTYPE);
