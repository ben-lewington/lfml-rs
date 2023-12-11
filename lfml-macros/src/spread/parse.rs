use proc_macro2::{Delimiter, Ident, TokenTree};
use syn::{
    spanned::Spanned, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit,
    Field, Fields, FieldsNamed, Lit, Meta, MetaList, MetaNameValue, Type, TypePath, Variant,
};

use crate::spread::{
    syntax::{ImplTags, SpreadBlock, SpreadData, SpreadField, SpreadInput},
    DATA_PREFIX,
};

impl SpreadInput {
    pub fn parse(
        DeriveInput {
            attrs,
            vis: _,
            ident: struct_ident,
            generics,
            data,
        }: syn::DeriveInput,
    ) -> syn::Result<Self> {
        let mut tags = ImplTags::DefaultWith {
            include: None,
            exclude: None,
        };
        let mut prefix: Option<String> = None;
        let mut suffix: Option<String> = None;

        for Attribute {
            pound_token: _,
            style,
            bracket_token: _,
            meta,
        } in attrs
        {
            match (style, &meta) {
                (AttrStyle::Outer, syn::Meta::Path(p))
                    if p.get_ident().filter(|p| *p == "prefix").is_some() =>
                {
                    prefix.replace(DATA_PREFIX.into());
                }
                (
                    AttrStyle::Outer,
                    syn::Meta::NameValue(MetaNameValue {
                        path: p,
                        eq_token: _,
                        value,
                    }),
                ) if p.get_ident().filter(|p| *p == "prefix").is_some() => {
                    let Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(s),
                    }) = value
                    else {
                        return Err(syn::Error::new(p.span(), "Expected string literal value"));
                    };
                    prefix.replace(s.value());
                }
                (
                    AttrStyle::Outer,
                    syn::Meta::NameValue(MetaNameValue {
                        path: p,
                        eq_token: _,
                        value,
                    }),
                ) if p.get_ident().filter(|p| *p == "suffix").is_some() => {
                    let Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(s),
                    }) = value
                    else {
                        return Err(syn::Error::new(p.span(), "Expected string literal value"));
                    };
                    suffix.replace(s.value());
                }
                (
                    AttrStyle::Outer,
                    syn::Meta::List(MetaList {
                        path: p,
                        delimiter: _,
                        tokens,
                    }),
                ) if p.get_ident().filter(|p| *p == "tags").is_some() => {
                    let mut it = tokens.clone().into_iter();

                    let mut include: Option<Vec<Ident>> = None;
                    let mut exclude: Option<Vec<Ident>> = None;
                    let mut only: Option<Vec<Ident>> = None;

                    loop {
                        match it.next() {
                            Some(TokenTree::Ident(i))
                                if i == "include" || i == "exclude" || i == "only" =>
                            {
                                match it.next() {
                                    Some(TokenTree::Group(g))
                                        if g.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        let mut v = vec![];
                                        for t in g.stream().into_iter() {
                                            match t {
                                                TokenTree::Ident(i) => v.push(i),
                                                TokenTree::Punct(p) if p.as_char() == ',' => {}
                                                _ => {
                                                    return Err(syn::Error::new(
                                                        i.span(),
                                                        "expected a comma separated list of idents",
                                                    ))
                                                }
                                            }
                                        }
                                        if i == "include" {
                                            include.replace(v);
                                        } else if i == "exclude" {
                                            exclude.replace(v);
                                        } else if i == "only" {
                                            only.replace(v);
                                        }
                                    }
                                    Some(TokenTree::Punct(_)) => {}
                                    None => break,
                                    t => {
                                        return Err(syn::Error::new(
                                            i.span(),
                                            format!("expected list of tag names, got {t:?}"),
                                        ))
                                    }
                                }
                            }
                            None => break,
                            _ => {
                                return Err(syn::Error::new(
                                    p.span(),
                                    "expected either include, exclude or only idents",
                                ))
                            }
                        }
                    }

                    tags = match (include, exclude, only) {
                        (None, None, None) => tags,
                        (None, None, Some(only)) => ImplTags::Only(only),
                        (include, exclude, None) => ImplTags::DefaultWith { include, exclude },
                        _ => {
                            return Err(syn::Error::new(
                                p.span(),
                                "either only attr alone, or include and/or exclude",
                            ))
                        }
                    }
                }
                _ => return Err(syn::Error::new(meta.span(), "unexpected attribute")),
            }
        }

        let fields = match data {
            Data::Struct(DataStruct {
                struct_token: _,
                fields:
                    Fields::Named(FieldsNamed {
                        brace_token: _,
                        named: fields,
                    }),
                semi_token: _,
            }) => {
                let mut fs = vec![];

                for Field {
                    attrs,
                    vis: _,
                    mutability: _,
                    ident,
                    colon_token: _,
                    ty,
                } in fields
                {
                    let is_option = if let Type::Path(TypePath { qself: _, path }) = ty {
                        path.segments
                            .iter()
                            .last()
                            .filter(|ps| ps.ident == "Option")
                            .is_some()
                    } else {
                        false
                    };

                    let mut is_escaped = false;
                    let mut rename = None;
                    for Attribute {
                        pound_token: _,
                        style,
                        bracket_token: _,
                        meta,
                    } in attrs
                    {
                        let span = meta.span();
                        match (style, meta.clone()) {
                            (AttrStyle::Outer, Meta::Path(p))
                                if p.get_ident().filter(|p| *p == "escape_value").is_some() =>
                            {
                                is_escaped = true;
                            }
                            (
                                AttrStyle::Outer,
                                Meta::NameValue(MetaNameValue {
                                    path: p,
                                    eq_token: _,
                                    value,
                                }),
                            ) if p.get_ident().filter(|p| *p == "rename").is_some() => {
                                let Expr::Lit(ExprLit {
                                    attrs: _,
                                    lit: Lit::Str(s),
                                }) = value
                                else {
                                    return Err(syn::Error::new(
                                        p.span(),
                                        "Expected string literal value",
                                    ));
                                };
                                rename.replace(s.value());
                            }
                            (_, _) => return Err(syn::Error::new(span, "unexpected attribute")),
                        }
                    }

                    fs.push(SpreadField {
                        rename,
                        name: ident.expect("named field"),
                        is_option,
                        is_escaped,
                    });
                }

                SpreadData::Struct(SpreadBlock {
                    prefix,
                    suffix,
                    fields: fs,
                })
            }
            Data::Enum(DataEnum {
                enum_token: _,
                brace_token: _,
                variants,
            }) => {
                let mut vars = vec![];

                for Variant {
                    attrs,
                    ident: var_ident,
                    fields,
                    discriminant: _,
                } in variants.iter()
                {
                    for Attribute {
                        pound_token: _,
                        style,
                        bracket_token: _,
                        meta,
                    } in attrs
                    {
                        match (style, &meta) {
                            (AttrStyle::Outer, syn::Meta::Path(p))
                                if p.get_ident().filter(|p| *p == "prefix").is_some() =>
                            {
                                prefix.replace(DATA_PREFIX.into());
                            }
                            (
                                AttrStyle::Outer,
                                syn::Meta::NameValue(MetaNameValue {
                                    path: p,
                                    eq_token: _,
                                    value,
                                }),
                            ) if p.get_ident().filter(|p| *p == "prefix").is_some() => {
                                let Expr::Lit(ExprLit {
                                    attrs: _,
                                    lit: Lit::Str(s),
                                }) = value
                                else {
                                    return Err(syn::Error::new(
                                        p.span(),
                                        "Expected string literal value",
                                    ));
                                };
                                prefix.replace(s.value());
                            }
                            (
                                AttrStyle::Outer,
                                syn::Meta::NameValue(MetaNameValue {
                                    path: p,
                                    eq_token: _,
                                    value,
                                }),
                            ) if p.get_ident().filter(|p| *p == "suffix").is_some() => {
                                let Expr::Lit(ExprLit {
                                    attrs: _,
                                    lit: Lit::Str(s),
                                }) = value
                                else {
                                    return Err(syn::Error::new(
                                        p.span(),
                                        "Expected string literal value",
                                    ));
                                };
                                suffix.replace(s.value());
                            }
                            _ => return Err(syn::Error::new(meta.span(), "unexpected attribute")),
                        }
                    }
                    let Fields::Named(FieldsNamed {
                        brace_token: _,
                        named: fields,
                    }) = fields
                    else {
                        return Err(syn::Error::new(
                            var_ident.span(),
                            "Only variants with named fields are supported",
                        ));
                    };

                    let mut fs = vec![];

                    for Field {
                        attrs,
                        vis: _,
                        mutability: _,
                        ident,
                        colon_token: _,
                        ty,
                    } in fields
                    {
                        let is_option = if let Type::Path(TypePath { qself: _, path }) = ty {
                            path.segments
                                .iter()
                                .last()
                                .filter(|ps| ps.ident == "Option")
                                .is_some()
                        } else {
                            false
                        };

                        let mut is_escaped = false;
                        let mut rename = None;
                        for Attribute {
                            pound_token: _,
                            style,
                            bracket_token: _,
                            meta,
                        } in attrs
                        {
                            match (style, meta) {
                                (AttrStyle::Outer, Meta::Path(p))
                                    if p.get_ident().filter(|p| *p == "escape_value").is_some() =>
                                {
                                    is_escaped = true;
                                }
                                (
                                    AttrStyle::Outer,
                                    Meta::NameValue(MetaNameValue {
                                        path: p,
                                        eq_token: _,
                                        value,
                                    }),
                                ) if p.get_ident().filter(|p| *p == "rename").is_some() => {
                                    let Expr::Lit(ExprLit {
                                        attrs: _,
                                        lit: Lit::Str(s),
                                    }) = value
                                    else {
                                        return Err(syn::Error::new(
                                            p.span(),
                                            "Expected string literal value",
                                        ));
                                    };
                                    rename.replace(s.value());
                                }
                                (_, _) => {
                                    return Err(syn::Error::new(
                                        meta.span(),
                                        "unexpected attribute",
                                    ))
                                }
                            }
                        }

                        fs.push(SpreadField {
                            rename,
                            name: ident.clone().expect("named field"),
                            is_option,
                            is_escaped,
                        });
                    }
                    vars.push((
                        var_ident.clone(),
                        SpreadBlock {
                            prefix: prefix.clone(),
                            suffix: suffix.clone(),
                            fields: fs,
                        },
                    ));
                }
                SpreadData::Enum(vars)
            }
            Data::Union(_) => todo!("unions can be implemented"),
            _ => todo!("unimplemented combination"),
        };

        Ok(SpreadInput {
            tags,
            fields,
            generics,
            r#struct: struct_ident,
        })
    }
}
