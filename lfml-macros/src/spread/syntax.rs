use proc_macro2::Ident;
use syn::Generics;

#[derive(Debug, Clone)]
pub struct SpreadInput {
    pub tags: ImplTags,
    pub r#struct: Ident,
    pub fields: SpreadData,
    pub generics: Generics,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SpreadData {
    Struct(SpreadBlock),
    Enum(Vec<(Ident, SpreadBlock)>),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImplTags {
    DefaultWith {
        include: Option<Vec<Ident>>,
        exclude: Option<Vec<Ident>>,
    },
    Only(Vec<Ident>),
}

#[derive(Debug, Clone)]
pub struct SpreadBlock {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub fields: Vec<SpreadField>,
}

#[derive(Debug, Clone)]
pub struct SpreadField {
    pub rename: Option<String>,
    pub name: Ident,
    pub is_option: bool,
    pub is_escaped: bool,
}
