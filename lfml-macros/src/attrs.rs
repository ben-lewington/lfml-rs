use quote::quote;
use syn::{
    AttrStyle, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsNamed,
    GenericParam, Lit, LitStr, Meta, MetaNameValue, TypeParam,
};

pub fn expand_embed_as_attrs(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
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
            Meta::Path(p) => {
                let Some(pfx) = p.get_ident() else {
                    return None;
                };
                if pfx == "prefix" {
                    Some("data".to_owned())
                } else {
                    None
                }
            }
            Meta::NameValue(MetaNameValue {
                path,
                eq_token: _,
                value,
            }) => {
                let Some(pfx) = path.get_ident() else {
                    return None;
                };

                if pfx == "prefix" {
                    // TODO: it's important that the value is a LitStr
                    let Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(l),
                    }) = value
                    else {
                        return None;
                    };

                    Some(l.value())
                } else {
                    None
                }
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
                path,
                eq_token: _,
                value,
            }) => {
                let Some(pfx) = path.get_ident() else {
                    return None;
                };

                if pfx == "suffix" {
                    // TODO: it's important that the value is a LitStr
                    let Expr::Lit(ExprLit {
                        attrs: _,
                        lit: Lit::Str(l),
                    }) = value
                    else {
                        return None;
                    };

                    Some(l.value())
                } else {
                    None
                }
            }
            Meta::Path(_) => None,
            Meta::List(_) => None,
        }
    });

    let disp_where: Vec<proc_macro2::TokenStream> = generics
        .params
        .iter()
        .filter_map(|gp| {
            let GenericParam::Type(TypeParam {
                attrs: _,
                ident,
                colon_token: _,
                bounds: _,
                eq_token: _,
                default: _,
            }) = gp
            else {
                return None;
            };
            Some(quote! { where #ident: ::core::fmt::Display })
        })
        .collect();

    // TODO: for any type parameters, when we implement EmbedAsAttrs, we will need to add a
    // core::fmt::Display trait bound
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
            ident.span(),
            "Currently only structs with named fields can derive EmbedAsAttrs.",
        ));
    };

    let Fields::Named(FieldsNamed {
        brace_token: _,
        named,
    }) = fields
    else {
        return Err(syn::Error::new(
            ident.span(),
            "Currently only structs with named fields can derive EmbedAsAttrs",
        ));
    };

    let mut format_string = String::with_capacity(
        named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string().chars().count() + 6)
            .sum::<usize>()
            + 1,
    );

    format_string.push(' ');

    let mut fields_pfx = Vec::with_capacity(named.iter().count());

    for Field {
        attrs,
        vis: _,
        mutability: _,
        ident,
        colon_token: _,
        ty: _,
    } in named
    {
        let ident = ident.as_ref().unwrap();

        if let Some(_escape) = attrs.iter().find(|&a| {
            if let Attribute {
                pound_token: _,
                style: AttrStyle::Outer,
                bracket_token: _,
                meta: Meta::Path(path),
            } = a
            {
                let Some(i) = path.get_ident() else {
                    return false;
                };

                i == "escape_value"
            } else {
                false
            }
        }) {
            fields_pfx.push(quote! {
                lfml::escape_string(&self.#ident.to_string())
            });
        } else {
            fields_pfx.push(quote! {
                self.#ident
            });
        };

        let to_attr = if let Some(t) = attrs.iter().find_map(|a| {
            if let Attribute {
                pound_token: _,
                style: AttrStyle::Outer,
                bracket_token: _,
                meta,
            } = a
            {
                let Meta::NameValue(MetaNameValue {
                    path,
                    eq_token: _,
                    value,
                }) = meta
                else {
                    return None;
                };
                if let Some(p) = path.get_ident() {
                    if p == "rename" {
                        let Expr::Lit(ExprLit {
                            attrs: _,
                            lit: Lit::Str(l),
                        }) = value
                        else {
                            return None;
                        };
                        return Some(l.value());
                    }
                } else {
                    return None;
                };
            };
            None
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

        format_string.push_str(&format!("{}=\"{{}}\" ", to_attr));
    }

    let fmt_str = LitStr::new(&format_string, ident.span());

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics lfml::EmbedAsAttrs for #ident #impl_ty #impl_where #(#disp_where),* {
            fn raw(&self) -> String {
                format!(#fmt_str, #(#fields_pfx),*)
            }
        }
    })
}
