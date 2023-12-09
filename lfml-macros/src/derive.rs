use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    AttrStyle, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsNamed,
    GenericParam, Lit, LitStr, Meta, MetaList, MetaNameValue, Type, TypeParam, TypePath,
};

use crate::html::VALID_HTML5_TAGS;

pub fn expand_embed_as_attrs(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident: struct_ident,
        generics,
        data,
    } = input;

    let global_pfx = attrs.iter().find_map(|a| {
        let Attribute {
            pound_token: _,
            style: AttrStyle::Outer,
            bracket_token: _,
            meta,
        } = a
        else {
            return None;
        };

        match meta {
            Meta::Path(p) => Some(
                p.get_ident()
                    .filter(|p| *p == "prefix")
                    .map(|_| "data".to_owned())?,
            ),
            Meta::NameValue(MetaNameValue {
                path: p,
                eq_token: _,
                value,
            }) => {
                let Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(l),
                }) = value
                else {
                    return None;
                };

                Some(
                    p.get_ident()
                        .filter(|p| *p == "prefix")
                        .map(|_| l.value())?,
                )
            }
            Meta::List(_) => None,
        }
    });

    let global_sfx = attrs.iter().find_map(|a| {
        let Attribute {
            pound_token: _,
            style: AttrStyle::Outer,
            bracket_token: _,
            meta,
        } = a
        else {
            return None;
        };
        match meta {
            Meta::NameValue(MetaNameValue {
                path: p,
                eq_token: _,
                value,
            }) => {
                // TODO: it's important that the value is a LitStr
                let Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(l),
                }) = value
                else {
                    return None;
                };

                Some(
                    p.get_ident()
                        .filter(|p| *p == "suffix")
                        .map(|_| l.value())?,
                )
            }
            Meta::Path(_) => None,
            Meta::List(_) => None,
        }
    });

    let tags = attrs
        .iter()
        .find_map(|a| {
            let Attribute {
                pound_token: _,
                style: AttrStyle::Outer,
                bracket_token: _,
                meta,
            } = a
            else {
                return None;
            };
            match meta {
                Meta::List(MetaList {
                    path: p,
                    delimiter: _,
                    tokens,
                }) => {
                    p.get_ident().filter(|p| *p == "tags")?;

                    Some(
                        tokens
                            .clone()
                            .into_iter()
                            .filter_map(|t| {
                                let proc_macro2::TokenTree::Ident(i) = t else {
                                    return None;
                                };

                                let impl_i = format_ident!("__lfml_tag_{i}");
                                Some(quote! {
                                    fn #impl_i(&self) -> String {
                                        lfml::MarkupAttrs::raw(&self)
                                    }
                                })
                            })
                            .collect::<Vec<_>>(),
                    )
                }
                Meta::Path(_) => None,
                _ => None,
            }
        })
        .unwrap_or(
            VALID_HTML5_TAGS
                .iter()
                .map(|t| {
                    let i = Ident::new(t, struct_ident.span());

                    let impl_i = format_ident!("__lfml_tag_{i}");
                    quote! {
                        fn #impl_i(&self) -> String {
                            lfml::MarkupAttrs::raw(&self)
                        }
                    }
                })
                .collect(),
        );

    let disp_where: Vec<proc_macro2::TokenStream> = generics
        .params
        .iter()
        .filter_map(|gp| {
            if let GenericParam::Type(TypeParam {
                attrs: _,
                ident,
                colon_token: _,
                bounds: _,
                eq_token: _,
                default: _,
            }) = gp
            {
                Some(quote! { where #ident: ::core::fmt::Display })
            } else {
                None
            }
        })
        .collect();

    let (impl_generics, impl_ty, impl_where) = generics.split_for_impl();

    let Data::Struct(DataStruct {
        struct_token: _,
        fields,
        semi_token: _,
    }) = data
    else {
        // TODO: as long as the fields are named (or all unnamed fields have an attr), we can
        // handle them
        return Err(syn::Error::new(
            struct_ident.span(),
            "Currently only structs with named fields can derive MarkupAttrs.",
        ));
    };

    let Fields::Named(FieldsNamed {
        brace_token: _,
        named,
    }) = fields
    else {
        return Err(syn::Error::new(
            struct_ident.span(),
            "Currently only structs with named fields can derive MarkupAttrs",
        ));
    };

    let mut format_string = String::with_capacity(
        named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string().chars().count() + 6)
            .sum::<usize>()
            + 1,
    );

    let mut fields_pfx = Vec::with_capacity(named.iter().count());

    for Field {
        attrs,
        vis: _,
        mutability: _,
        ident,
        colon_token: _,
        ty,
    } in named
    {
        let ident = ident.as_ref().unwrap();

        let is_option_type = if let Type::Path(TypePath { qself: _, path }) = ty {
            path.segments
                .iter()
                .last()
                .filter(|ps| ps.ident == "Option")
                .is_some()
        } else {
            false
        };

        let is_escape_value = attrs.iter().any(|a| {
            if let Attribute {
                pound_token: _,
                style: AttrStyle::Outer,
                bracket_token: _,
                meta: Meta::Path(path),
            } = a
            {
                path.get_ident().filter(|i| *i == "escape_value").is_some()
            } else {
                false
            }
        });

        let field_attr_name = if let Some(t) = attrs.iter().find_map(|a| {
            if let Attribute {
                pound_token: _,
                style: AttrStyle::Outer,
                bracket_token: _,
                meta:
                    Meta::NameValue(MetaNameValue {
                        path,
                        eq_token: _,
                        value:
                            Expr::Lit(ExprLit {
                                attrs: _,
                                lit: Lit::Str(l),
                            }),
                    }),
            } = a
            {
                Some(
                    path.get_ident()
                        .filter(|p| *p == "rename")
                        .map(|_| l.value())?,
                )
            } else {
                None
            }
        }) {
            t.to_string()
        } else {
            let t = if let Some(ref p) = global_pfx {
                // TODO: a guard on the length of p > 0? maybe that lives upstairs
                format!("{p}-{}", ident)
            } else {
                ident.to_string()
            };

            if let Some(ref s) = global_sfx {
                // TODO: a guard on the length of p > 0? maybe that lives upstairs
                format!("{}-{s}", t)
            } else {
                t
            }
        };

        let fmt_attr = format!(" {}=\"{{}}\"", field_attr_name);

        let fmt_expr = if is_option_type {
            let fmt_attr_lit = LitStr::new(&fmt_attr, ident.span());

            let fmt_value = if is_escape_value {
                quote! { lfml::escape_string(&x.to_string()) }
            } else {
                quote! { &x }
            };

            quote! {{
                if let Some(ref x) = self.#ident {
                    format!(#fmt_attr_lit, #fmt_value)
                } else {
                    "".into()
                }
            }}
        } else if is_escape_value {
            quote! { lfml::escape_string(&self.#ident.to_string()) }
        } else {
            quote! { self.#ident }
        };

        fields_pfx.push(fmt_expr);

        format_string.push_str(if is_option_type { "{}" } else { &fmt_attr });
    }

    let fmt_str = LitStr::new(&format_string, struct_ident.span());

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics lfml::MarkupAttrs for #struct_ident #impl_ty #impl_where #(#disp_where),* {
            fn raw(&self) -> String {
                format!(#fmt_str, #(#fields_pfx),*)
            }
        }

        impl #impl_generics #struct_ident #impl_ty #impl_where #(#disp_where),* {
            #(#tags)*
        }
    })
}
