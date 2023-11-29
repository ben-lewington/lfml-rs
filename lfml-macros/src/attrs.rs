use quote::quote;
use syn::{
    AttrStyle, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsNamed,
    Lit, LitStr, Meta, MetaNameValue,
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
                if pfx.to_string() == "prefix" {
                    return Some("data".to_owned());
                } else {
                    return None;
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

                if pfx.to_string() == "prefix" {
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
                    return None;
                }
            }
            Meta::List(_) => return None,
        }
    });

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
                return i.to_string() == "escape_value";
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

        let to_attr = if let Some(ref p) = global_pfx {
            // TODO: a guard on the length of p > 0? maybe that lives upstairs
            format!("{p}-{}", ident.to_string())
        } else {
            ident.to_string()
        };

        format_string.push_str(&format!("{}=\"{{}}\" ", to_attr));
    }

    let fmt_str = LitStr::new(&format_string, ident.span());

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics lfml::EmbedAsAttrs for #ident #impl_ty #impl_where {
            fn raw(&self) -> String {
                format!(#fmt_str, #(#fields_pfx),*)
            }
        }
    })
}
