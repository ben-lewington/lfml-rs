use crate::{html::VALID_HTML5_TAGS, spread::syntax::SpreadVariant};

use std::iter::Extend;

use super::syntax::{SpreadInput, SpreadFields, SpreadBlock, SpreadField};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, TokenStreamExt};
use syn::{GenericParam, TypeParam, LitStr};

pub fn generate_spread_impl(
    SpreadInput {
        tags,
        prefix: global_pfx,
        suffix: global_sfx,
        fields,
        generics,
        r#struct,
    }: SpreadInput,
    output: &mut proc_macro2::TokenStream,
) -> syn::Result<()> {
    let (impl_generics, impl_ty, impl_where) = generics.split_for_impl();

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

    let tag_wrapper = format_ident!("{struct}Tags");
    let tags: Vec<TokenStream> = match tags {
        super::syntax::ImplTags::DefaultWith { include, exclude } => {
            let mut ts = VALID_HTML5_TAGS
                .into_iter()
                .map(|&tag| Ident::new(tag, Span::mixed_site()))
                .filter(|t| {
                    exclude
                        .as_ref()
                        .filter(|e| e.iter().any(|e| e == t))
                        .is_none()
                        ||
                    include
                        .as_ref()
                        .filter(|e| e.iter().any(|e| e == t))
                        .is_some()
                })
                .collect::<Vec<_>>();

            if let Some(incl) = include {
                ts.extend(incl);
            }

            ts.into_iter()
                .map(|tag| {
                    quote! {
                        fn #tag(&self) -> String {
                            lfml::MarkupAttrs::raw(&self.0)
                        }
                    }
                })
                .collect()
        }
        super::syntax::ImplTags::Only(o) => o
            .into_iter()
            .map(|tag| {
                quote! {
                    fn #tag(&self) -> String {
                        lfml::MarkupAttrs::raw(&self.0)
                    }
                }
            })
            .collect(),
    };

    let impl_raw_body = match fields {
        SpreadFields::Struct(SpreadBlock {
            variant: None,
            fields,
        }) => {
            let mut fmt_value_exprs = vec![];
            let mut fmt_string = String::new();

            for SpreadField {
                rename,
                name,
                is_option,
                is_escaped,
            } in fields {
                let attribute_name = if let Some(t) = rename {
                    t.to_string()
                } else {
                    let t = if let Some(ref pfx) = global_pfx {
                        // TODO: a guard on the length of p > 0? maybe that lives upstairs
                        format!("{pfx}-{name}")
                    } else {
                        name.to_string()
                    };

                    if let Some(ref sfx) = global_sfx {
                        // TODO: a guard on the length of p > 0? maybe that lives upstairs
                        format!("{t}-{sfx}")
                    } else {
                        t
                    }
                };

                let fmt_attr = format!(" {}=\"{{}}\"", attribute_name);

                let escape_value = |val| if is_escaped {
                    quote! { lfml::escape_string(#val.to_string()) }
                } else {
                    quote! { #val }
                };


                let fmt_expr = if is_option {
                    let fmt_attr = LitStr::new(&fmt_attr, name.span());

                    let fmt_value = escape_value(quote! { x });
                    quote! {{
                        if let Some(ref x) = self.#name {
                            format!(#fmt_attr, #fmt_value)
                        } else {
                            "".into()
                        }
                    }}
                } else {
                    escape_value(quote! { self.#name })
                };

                fmt_value_exprs.push(fmt_expr);
                fmt_string.push_str(if is_option { "{}" } else { &fmt_attr });
            }
            quote! { format!(#fmt_string, #(#fmt_value_exprs),*) }
        },
        SpreadFields::Enum(var_blocks) => {
            let mut vars = vec![];

            for SpreadBlock {
                variant,
                fields
            } in var_blocks {
                let Some(SpreadVariant { prefix: var_pfx, suffix: var_sfx, name: var_name }) = variant else {
                    return Err(syn::Error::new(Span::mixed_site(), "expected variant attrs here"));
                };
                let mut fs = TokenStream::new();
                let mut fmt_value_exprs = vec![];
                let mut fmt_string = String::new();

                for SpreadField {
                    rename,
                    name,
                    is_option,
                    is_escaped,
                } in fields {
                    let attribute_name = if let Some(t) = rename {
                        t.to_string()
                    } else {
                        let t = if let Some(ref pfx) = var_pfx.clone().or_else(|| global_pfx.clone()) {
                            // TODO: a guard on the length of p > 0? maybe that lives upstairs
                            format!("{pfx}-{name}")
                        } else {
                            name.to_string()
                        };

                        if let Some(ref sfx) = var_sfx.clone().or_else(|| global_sfx.clone()) {
                            // TODO: a guard on the length of p > 0? maybe that lives upstairs
                            format!("{t}-{sfx}")
                        } else {
                            t
                        }
                    };

                    let fmt_attr = format!(" {}=\"{{}}\"", attribute_name);

                    let escape_value = |val| if is_escaped {
                        quote! { lfml::escape_string(#val.to_string()) }
                    } else {
                        quote! { #val }
                    };


                    let fmt_expr = if is_option {
                        let fmt_attr = LitStr::new(&fmt_attr, name.span());

                        let fmt_value = escape_value(quote! { x });
                        quote! {{
                            if let Some(ref x) = #name {
                                format!(#fmt_attr, #fmt_value)
                            } else {
                                "".into()
                            }
                        }}
                    } else {
                        escape_value(quote! { #name })
                    };

                    fmt_value_exprs.push(fmt_expr);
                    fmt_string.push_str(if is_option { "{}" } else { &fmt_attr });
                    fs.append_all(quote! { #name, });
                }
                vars.push(quote! {
                    Self::#var_name { #fs } => {
                        format!(#fmt_string, #(#fmt_value_exprs),*),
                    }
                });
            }
            quote! {
                match self {
                    #(#vars)*
                }
            }
        }
        _ => todo!(),
    };

    output.append_all(quote! {
        #[automatically_derived]
        impl #impl_generics lfml::MarkupAttrs for #r#struct #impl_ty #impl_where #(#disp_where),* {
            fn raw(&self) -> String {
                #impl_raw_body
            }
        }

        pub struct #tag_wrapper<T>(pub T);

        impl #impl_generics #tag_wrapper< &#r#struct #impl_ty > #impl_where #(#disp_where),* {
            #(#tags)*
        }

        impl #impl_generics #r#struct #impl_ty #impl_where #(#disp_where),* {
            fn __lfml_tags(&self) -> #tag_wrapper< &#r#struct #impl_ty > {
                #tag_wrapper (self)
            }
        }
    });

    Ok(())
}
