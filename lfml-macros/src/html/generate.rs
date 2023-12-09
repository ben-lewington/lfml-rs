use crate::html::syntax::{Interpolate, InterpolateWrapper, Markup, MarkupAttr};

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, TokenStreamExt};



pub fn markup_as_string_push_operations(
    buffer_id: &Ident,
    input: Vec<Markup>,
    output: &mut TokenStream,
) -> syn::Result<()> {
    for markup in input {
        match markup {
            Markup::LiteralSequence(ls) => {
                let mut lit_concat = String::new();
                for l in ls {
                    l.push_to_string(&mut lit_concat)?;
                }

                let litstr = Literal::string(&lit_concat);

                output.append_all(quote! {
                    #buffer_id.push_str(#litstr);
                });
            }
            Markup::MarkupTag {
                ident,
                attrs,
                inner,
            } => {
                let mut opening_tag = String::from("<");
                let mut interp_attrs = vec![];

                opening_tag.push_str(&ident.to_string());

                for attr in attrs {
                    match attr {
                        MarkupAttr::Single { name } => {
                            opening_tag.push(' ');
                            opening_tag.push_str(&name.to_string());
                        }
                        MarkupAttr::Static { name, value } => {
                            opening_tag.push_str(&format!(" {}=\"", name));
                            value.push_to_string(&mut opening_tag)?;
                            opening_tag.push('"');
                        }
                        MarkupAttr::Interpolate { value, r#type } => match r#type {
                            Interpolate::Toggle { name } => {
                                let litstr = Literal::string(&format!(" {name}"));
                                interp_attrs.push(quote! {
                                    if { #value } {
                                        #litstr
                                    } else {
                                        ""
                                    }
                                });
                                opening_tag.push_str("{}");
                            }
                            Interpolate::NameValue { name, wrapper } => match wrapper {
                                InterpolateWrapper::None => {
                                    interp_attrs.push(quote! {
                                        lfml::escape_string(&{
                                            #value
                                        }.to_string())
                                    });
                                    opening_tag.push_str(&format!(" {}=\"{{}}\"", name));
                                }
                                InterpolateWrapper::Option => {
                                    let litstr = Literal::string(&format!(" {}=\"{{}}\"", name));

                                    interp_attrs.push(quote! {
                                        if let Some(e) = { #value } {
                                            format!(#litstr, e)
                                        } else {
                                            "".into()
                                        }
                                    });

                                    opening_tag.push_str("{}");
                                }
                            },
                            Interpolate::Spread { tag, wrapper } => {
                                let tag = format_ident!("__lfml_tag_{tag}");
                                match wrapper {
                                    InterpolateWrapper::None => {
                                        interp_attrs.push(quote! { {
                                            { &#value }.#tag()
                                        }});
                                    }
                                    InterpolateWrapper::Option => {
                                        interp_attrs.push(quote! { {
                                            if let Some(i) = { &#value } {
                                                i.#tag()
                                            } else {
                                                "".into()
                                            }
                                        }});
                                    }
                                }

                                opening_tag.push_str("{}");
                            }
                        },
                    }
                }

                opening_tag.push('>');

                let open = Literal::string(&opening_tag);

                output.append_all(if interp_attrs.is_empty() {
                    quote! {
                        #buffer_id.push_str(#open);
                    }
                } else {
                    quote! {
                        #buffer_id.push_str(
                            &format!(#open, #(#interp_attrs),*)
                        );
                    }
                });

                if let Some(inner) = inner {
                    let close = Literal::string(&format!("</{}>", ident));

                    markup_as_string_push_operations(buffer_id, inner, output)?;

                    output.append_all(quote! {
                        #buffer_id.push_str(#close);
                    })
                }
            }
            Markup::AnonBlock(b) => {
                markup_as_string_push_operations(buffer_id, b, output)?;
            }
            Markup::Interpolated(i) => {
                output.append_all(quote! {
                    #buffer_id.push_str(&lfml::Render::markup(&{#i}).as_string());
                });
            }
        }
    }
    Ok(())
}
