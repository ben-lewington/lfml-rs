use crate::html::syntax::{InterpValue, InterpValueType, Markup, TagAttribute};

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, TokenStreamExt};

use super::syntax::{InterpMarkupExpr, MarkupId};

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
            Markup::Tag { tag, attrs, inner } => {
                let mut opening_tag = String::from("<");
                let mut interp_attrs = vec![];

                opening_tag.push_str(&tag.to_string());

                for attr in attrs {
                    match attr {
                        TagAttribute::Lit { name, value } => {
                            opening_tag.push(' ');
                            opening_tag.push_str(&name.to_string());
                            if let Some(v) = value {
                                opening_tag.push_str("=\"");
                                v.push_to_string(&mut opening_tag)?;
                                opening_tag.push('\"');
                            }
                        }
                        TagAttribute::Interpolated { value, r#type } => match r#type {
                            InterpValue::Toggle { name } => {
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
                            InterpValue::NameValue { name, wrapper } => match wrapper {
                                InterpValueType::None => {
                                    interp_attrs.push(quote! {
                                        lfml::escape_string(&{
                                            #value
                                        }.to_string())
                                    });
                                    opening_tag.push_str(&format!(" {}=\"{{}}\"", name));
                                }
                                InterpValueType::Option => {
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
                            InterpValue::Spread { tag, wrapper } => {
                                let MarkupId::Basic(tag) = tag else {
                                    todo!("spreading for tags containing hyphens");
                                };
                                match wrapper {
                                    InterpValueType::None => {
                                        interp_attrs.push(quote! { {
                                            { &#value }.__lfml_tags().#tag()
                                        }});
                                    }
                                    InterpValueType::Option => {
                                        interp_attrs.push(quote! { {
                                            if let Some(i) = { &#value } {
                                                i.__lfml_tags().#tag()
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
                    markup_as_string_push_operations(buffer_id, inner, output)?;

                    let close = Literal::string(&format!("</{}>", tag));
                    output.append_all(quote! {
                        #buffer_id.push_str(#close);
                    })
                }
            }
            Markup::AnonBlock(b) => {
                markup_as_string_push_operations(buffer_id, b, output)?;
            }
            Markup::Slot(InterpMarkupExpr::Simple(s)) => {
                output.append_all(quote! {
                    #buffer_id.push_str(&lfml::Render::markup(&{#s}).as_string());
                });
            }
            Markup::Slot(InterpMarkupExpr::Match(outer, variants)) => {
                let mut vars = vec![];
                for (pattern, value) in variants {
                    let mut value_expr = TokenStream::new();
                    markup_as_string_push_operations(buffer_id, value, &mut value_expr)?;

                    vars.push(quote! {
                        #pattern => { #value_expr },
                    });
                }
                output.append_all(quote! {
                    #outer {
                        #(#vars)*
                    }
                });
            }
            Markup::Slot(InterpMarkupExpr::For(outer, repeat_block)) => {
                let mut value_expr = TokenStream::new();
                markup_as_string_push_operations(buffer_id, repeat_block, &mut value_expr)?;
                output.append_all(quote! {
                    #outer {
                        #value_expr
                    }
                });
            }
            Markup::Slot(InterpMarkupExpr::If {
                if_block: (if_expr, if_value),
                else_blocks,
            }) => {
                let mut if_value_expr = TokenStream::new();
                markup_as_string_push_operations(buffer_id, if_value, &mut if_value_expr)?;
                let mut elses = vec![];
                for (else_block, else_value) in else_blocks {
                    let mut else_value_expr = TokenStream::new();
                    markup_as_string_push_operations(buffer_id, else_value, &mut else_value_expr)?;

                    elses.push(quote! {
                        #else_block {
                            #else_value_expr
                        }
                    });
                }

                output.append_all(quote! {
                    #if_expr {
                        #if_value_expr
                    }
                    #(#elses)*
                });
            }
        }
    }
    Ok(())
}
