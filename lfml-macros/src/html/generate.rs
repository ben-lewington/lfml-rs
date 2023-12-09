use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, TokenStreamExt};

use super::syntax::{MarkupAttrSyntax, MarkupSyntax, AttrInterpType, AttrInterpTransform};

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
                        MarkupAttrSyntax::Single { key_ident } => {
                            opening_tag.push(' ');
                            opening_tag.push_str(&key_ident.to_string());
                        }
                        MarkupAttrSyntax::Static { key_ident, value } => {
                            opening_tag.push_str(&format!(" {}=\"", key_ident));
                            value.push_to_string(&mut opening_tag)?;
                            opening_tag.push('"');
                        }
                        MarkupAttrSyntax::Interpolate { block, r#type } => match r#type {
                            AttrInterpType::Single { ident, transform } => {
                                match transform {
                                    AttrInterpTransform::Bool => {
                                        let litstr = Literal::string(&format!(" {ident}"));
                                        interp_attrs.push(quote! {
                                            if { #block } {
                                                #litstr
                                            } else {
                                                ""
                                            }
                                        });
                                        opening_tag.push_str("{}");

                                    },
                                    _ => unreachable!(),
                                }
                            }
                            AttrInterpType::KeyValue { ident, transform } => {
                                match transform {
                                    AttrInterpTransform::None => {
                                        interp_attrs.push(quote! {
                                            lfml::escape_string(&{
                                                #block
                                            }.to_string())
                                        });
                                        opening_tag.push_str(&format!(" {}=\"{{}}\"", ident));
                                    }
                                    AttrInterpTransform::Option => {
                                        let litstr = Literal::string(&format!(" {}=\"{{}}\"", ident));

                                        interp_attrs.push(quote! {
                                            if let Some(e) = { #block } {
                                                format!(#litstr, e)
                                            } else {
                                                "".into()
                                            }
                                        });

                                        opening_tag.push_str("{}");
                                    },
                                    AttrInterpTransform::Bool => unreachable!(),
                                }
                            }
                            AttrInterpType::Spread { tag, transform } => {
                                let ensure_tag = format_ident!("__lfml_tag_{tag}");
                                match transform {
                                    AttrInterpTransform::None => {
                                        interp_attrs.push(quote! { {
                                            { &#block }.#ensure_tag()
                                        }});
                                    },
                                    AttrInterpTransform::Option => {
                                        interp_attrs.push(quote! { {
                                            if let Some(i) = { &#block } {
                                                i.#ensure_tag()
                                            } else {
                                                "".into()
                                            }
                                        }});
                                    },
                                    AttrInterpTransform::Bool => unreachable!(),
                                }

                                opening_tag.push_str("{}");
                            }
                        },                     }
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
