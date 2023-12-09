use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, TokenStreamExt};

use super::syntax::{Interp, InterpUnwrap, MarkupAttrSyntax, MarkupSyntax};

pub fn markup_as_string_push_operations(
    buffer_id: &Ident,
    input: Vec<MarkupSyntax>,
    output: &mut TokenStream,
) -> syn::Result<()> {
    for markup in input {
        match markup {
            MarkupSyntax::LiteralSequence(ls) => {
                let mut lit_concat = String::new();
                for l in ls {
                    l.push_to_string(&mut lit_concat)?;
                }

                let litstr = Literal::string(&lit_concat);

                output.append_all(quote! {
                    #buffer_id.push_str(#litstr);
                });
            }
            MarkupSyntax::MarkupTag {
                ident,
                attrs,
                inner,
            } => {
                let mut opening_tag = String::from("<");
                let mut interp_attrs = vec![];

                opening_tag.push_str(&ident.to_string());

                for attr in attrs {
                    match attr {
                        MarkupAttrSyntax::Single { name } => {
                            opening_tag.push(' ');
                            opening_tag.push_str(&name.to_string());
                        }
                        MarkupAttrSyntax::Static { name, value } => {
                            opening_tag.push_str(&format!(" {}=\"", name));
                            value.push_to_string(&mut opening_tag)?;
                            opening_tag.push('"');
                        }
                        MarkupAttrSyntax::Interpolate { value, r#type } => match r#type {
                            Interp::Toggle { name } => {
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
                            Interp::KeyValue { name, unwrap } => match unwrap {
                                InterpUnwrap::None => {
                                    interp_attrs.push(quote! {
                                        lfml::escape_string(&{
                                            #value
                                        }.to_string())
                                    });
                                    opening_tag.push_str(&format!(" {}=\"{{}}\"", name));
                                }
                                InterpUnwrap::Option => {
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
                            Interp::Spread { tag, unwrap } => {
                                let tag = format_ident!("__lfml_tag_{tag}");
                                match unwrap {
                                    InterpUnwrap::None => {
                                        interp_attrs.push(quote! { {
                                            { &#value }.#tag()
                                        }});
                                    }
                                    InterpUnwrap::Option => {
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
            MarkupSyntax::AnonBlock(b) => {
                markup_as_string_push_operations(buffer_id, b, output)?;
            }
            MarkupSyntax::Interpolated(i) => {
                output.append_all(quote! {
                    #buffer_id.push_str(&lfml::Render::markup(&{#i}).as_string());
                });
            }
        }
    }
    Ok(())
}
