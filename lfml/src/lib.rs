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

#[macro_export]
macro_rules! template {
    ($pub:vis $name:ident $inner:tt) => {
        $pub fn $name() -> lfml::Markup {
            lfml::html! { $inner }
        }
    };
    ($pub:vis $name:ident<$lt:tt> $inner:tt) => {
        $pub fn $name<$lt>() -> lfml::Markup {
            lfml::html! { $inner }
        }
    };
    ($pub:vis $name:ident$(<$lt:tt>)?($($arg:ident: $ty:ty),*) $inner:tt) => {
        $pub fn $name$(<$lt>)?($($arg: $ty),*) -> lfml::Markup {
            lfml::html! { $inner }
        }
    };
}
