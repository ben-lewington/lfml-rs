use crate::spread::syntax::{ImplTags, SpreadBlock, SpreadData, SpreadField, SpreadInput};

use lfml_html5::VALID_HTML5_TAGS;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{GenericParam, LitStr, TypeParam};

pub fn generate_spread_impl(
    SpreadInput {
        tags,
        data_id,
        fields,
        generics,
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

    let tag_wrapper = format_ident!("{data_id}Tags");
    let tags: Vec<TokenStream> = match tags {
        ImplTags::DefaultWith { include, exclude } => VALID_HTML5_TAGS
            .iter()
            .map(|&tag| Ident::new(tag, Span::mixed_site()))
            .filter(|t| {
                exclude
                    .as_ref()
                    .filter(|e| e.iter().any(|e| e == t))
                    .is_none()
            })
            .filter(|t| {
                include
                    .as_ref()
                    .filter(|e| e.iter().any(|e| e == t))
                    .is_none()
            })
            .chain(include.clone().unwrap_or(vec![]))
            .map(|tag| {
                quote! {
                    pub fn #tag(&self) -> String {
                        lfml::Spread::raw(&self.0)
                    }
                }
            })
            .collect::<Vec<_>>(),
        ImplTags::Only(o) => o
            .into_iter()
            .map(|tag| {
                quote! {
                    pub fn #tag(&self) -> String {
                        lfml::Spread::raw(&self.0)
                    }
                }
            })
            .collect(),
    };

    let impl_raw_body = match fields {
        SpreadData::Struct(block) => block.generate_tokens(None),
        SpreadData::Enum(var_blocks) => {
            let mut vars = vec![];

            for (var_name, block) in var_blocks {
                vars.push(block.generate_tokens(Some(var_name)));
            }
            quote! {
                match self {
                    #(#vars)*
                }
            }
        }
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics lfml::Spread for #data_id #impl_ty #impl_where #(#disp_where),* {
            fn raw(&self) -> String {
                #impl_raw_body
            }
        }

        pub struct #tag_wrapper<T>(pub T);

        impl #impl_generics #tag_wrapper< &#data_id #impl_ty > #impl_where #(#disp_where),* {
            #(#tags)*
        }

        impl #impl_generics #data_id #impl_ty #impl_where #(#disp_where),* {
            pub fn __lfml_tags(&self) -> #tag_wrapper< &#data_id #impl_ty > {
                #tag_wrapper (self)
            }
        }
    }
    .to_tokens(output);

    Ok(())
}

impl SpreadBlock {
    fn generate_tokens(&self, var_name: Option<Ident>) -> TokenStream {
        let mut fs = TokenStream::new();
        let mut fmt_value_exprs = vec![];
        let mut fmt_string = String::new();

        for SpreadField {
            rename,
            name,
            is_option,
            is_escaped,
            is_name_only,
        } in self.fields.clone().into_iter()
        {
            let attribute_name = if let Some(t) = rename {
                t.to_string()
            } else {
                let t = if let Some(pfx) = self.prefix.as_ref() {
                    // TODO: a guard on the length of p > 0? maybe that lives upstairs
                    format!("{pfx}-{name}")
                } else {
                    name.to_string()
                };

                if let Some(sfx) = self.suffix.as_ref() {
                    // TODO: a guard on the length of p > 0? maybe that lives upstairs
                    format!("{t}-{sfx}")
                } else {
                    t
                }
            };

            let fmt_attr = if !is_name_only {
                format!(" {}=\"{{}}\"", attribute_name)
            } else {
                format!(" {}", attribute_name)
            };

            let field_ident = if var_name.is_some() {
                quote! { &#name }
            } else {
                quote! { &self.#name }
            };

            let escape_value = |val| {
                if is_escaped {
                    quote! { lfml::escape_string( &#val.to_string() ) }
                } else {
                    quote! { #val }
                }
            };

            let fmt_expr = if is_option {
                let fmt_attr = LitStr::new(&fmt_attr, name.span());

                let fmt_value = escape_value(quote! { x });

                if !is_name_only {
                    Some(quote! { &{
                        if let Some(ref x) = #field_ident {
                            format!(#fmt_attr, #fmt_value)
                        } else {
                            "".into()
                        }
                    }})
                } else {
                    Some(quote! {
                        if let Some(_) = #field_ident {
                            #fmt_attr
                        } else {
                            ""
                        }
                    })
                }
            } else {
                if !is_name_only {
                    Some(escape_value(quote! { &{ #field_ident } }))
                } else {
                    None
                }
            };

            if let Some(fmt_expr) = fmt_expr {
                fmt_value_exprs.push(fmt_expr);
            }
            fmt_string.push_str(if is_option { "{}" } else { &fmt_attr });
            quote! { #name, }.to_tokens(&mut fs);
        }
        let fmt_lit = LitStr::new(&fmt_string, Span::mixed_site());

        match var_name {
            Some(var) => quote! {
                Self::#var { #fs } => {
                    format!(#fmt_lit, #(#fmt_value_exprs),*)
                }
            },
            None => quote! {
                format!(#fmt_lit, #(#fmt_value_exprs),*)
            },
        }
    }
}
