use proc_macro2::Ident;
use syn::Generics;


#[derive(Debug, Clone)]
pub struct SpreadInput {
    pub tags: ImplTags,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub r#struct: Ident,
    pub fields: SpreadFields,
    pub generics: Generics,
}

#[derive(Debug, Clone)]
pub enum SpreadFields {
    Struct(SpreadBlock),
    Enum(Vec<SpreadBlock>),
}

#[derive(Debug, Clone)]
pub enum ImplTags {
    DefaultWith {
        include: Option<Vec<Ident>>,
        exclude: Option<Vec<Ident>>,
    },
    Only(Vec<Ident>),
}

#[derive(Debug, Clone)]
pub struct SpreadBlock {
    pub variant: Option<SpreadVariant>,
    pub fields: Vec<SpreadField>,
}

#[derive(Debug, Clone)]
pub struct SpreadVariant {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub name: Ident,
}

#[derive(Debug, Clone)]
pub struct SpreadField {
    pub rename: Option<String>,
    pub name: Ident,
    pub is_option: bool,
    pub is_escaped: bool,
}
