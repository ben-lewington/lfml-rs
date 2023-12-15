use proc_macro2::Ident;
use syn::Generics;

#[derive(Debug, Clone)]
pub struct SpreadInput {
    /// Represents the HTML tags you can spread the data with.
    pub tags: ImplTags,
    /// The name of the struct or enum.
    pub data_id: Ident,
    /// Generics data, required for the output TokenStream.
    pub generics: Generics,
    /// Field names for writing out the Spread implementation.
    pub fields: SpreadData,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImplTags {
    /// This will allow the data to be spread with any HTML5 tags.
    ///
    /// Optionally, lists of tags to be excluded or included from the default can be specified:
    /// ```ignore
    /// #[derive(Spread)]
    /// #[tags(include(custom), exclude(img, link, script))]
    /// struct Foo {
    ///     bar: usize
    /// }
    /// ```
    /// Omission of the tags attribute is the same as:
    /// ```ignore
    /// ImplTags::DefaultWith { include: None, exclude:None}
    /// ```
    DefaultWith {
        include: Option<Vec<Ident>>,
        exclude: Option<Vec<Ident>>,
    },
    /// Use a provided a list of tags for spread:
    /// ```ignore
    /// #[derive(Spread)]
    /// #[tags(only(a))]
    /// struct Foo {
    ///     bar: usize
    /// }
    /// ```
    Only(Vec<Ident>),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SpreadData {
    /// data is a struct.
    Struct(SpreadBlock),
    /// data is an enum.
    Enum(Vec<(Ident, SpreadBlock)>),
}

#[derive(Debug, Clone)]
pub struct SpreadBlock {
    /// If there's a prefix attribute on either a struct or an enum variant, this will take that
    /// value. in the enum case, a prefix attribute on a variant will override a prefix on the enum.
    /// ```ignore
    /// #[derive(Spread)]
    /// #[prefix]
    /// struct Foo {
    ///     a: usize,
    /// }
    ///
    /// #[derive(Spread)]
    /// #[prefix]
    /// enum Foo {
    ///     Bar {
    ///         bar: usize,
    ///     },
    ///     #[prefix("x-data")]
    ///     Baz {
    ///         bar: usize,
    ///     }
    /// }
    /// ```
    /// The acceptable forms are either #[prefix] (defaults to "data"), or #[prefix = "foo"]
    pub prefix: Option<String>,
    /// If there's a suffix attribute on either a struct or an enum variant, this will take that
    /// value. in the enum case, a suffix attribute on a variant will override a prefix on the enum.
    /// ```ignore
    ///
    /// #[derive(lfml::Spread)]
    /// #[suffix = ""]
    /// struct Foo {
    ///     a: usize,
    /// }
    ///
    /// #[derive(lfml::Spread)]
    /// #[prefix]
    /// enum Bar {
    ///     Baz {
    ///         bat: usize,
    ///     },
    ///     #[prefix("x-data")]
    ///     Car {
    ///         cat: usize,
    ///     }
    /// }
    /// ```
    /// the acceptable forms is #[suffix = "foo"]
    pub suffix: Option<String>,
    /// Field names and flaggs.
    pub fields: Vec<SpreadField>,
}

#[derive(Debug, Clone)]
pub struct SpreadField {
    pub rename: Option<String>,
    pub name: Ident,
    pub is_option: bool,
    pub is_escaped: bool,
}
