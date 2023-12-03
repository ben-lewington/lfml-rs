use std::iter::Peekable;

use proc_macro2::{
    token_stream::IntoIter, Delimiter, Ident, Literal, Span, TokenStream, TokenTree,
};
use quote::quote;
use syn::Lit;

const SIZE_MULTIPLIER: usize = 10;

pub fn generate_markup_expr(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let out_id = Ident::new("__lfml_output", Span::mixed_site());
    let size_hint = input.to_string().len() * SIZE_MULTIPLIER;

    let mut output = vec![];

    process_tokens(input.into_iter().peekable(), &mut output, &out_id)?;

    Ok(quote! {{
        let mut #out_id = String::with_capacity(#size_hint);
        #(#output)*
        lfml::Escaped(#out_id)
    }})
}

fn process_tokens(
    mut tokens: Peekable<IntoIter>,
    output: &mut Vec<TokenStream>,
    out_id: &Ident,
) -> syn::Result<()> {
    'main: while let Some(t) = tokens.next() {
        let current_span = t.span();
        match t {
            TokenTree::Literal(l) => match Lit::new(l) {
                Lit::Str(s) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&s.value()));
                    output.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::ByteStr(bs) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(
                        &String::from_utf8_lossy(&bs.value()),
                    ));
                    output.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Byte(b) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&String::from(
                        b.value() as char,
                    )));
                    output.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Char(c) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&String::from(
                        c.value()
                    )));
                    output.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Int(i) => output.push(quote! {
                    #out_id.push_str(&#i.to_string());
                }),
                Lit::Float(lf) => output.push(quote! {
                    #out_id.push_str(&#lf.to_string());
                }),
                Lit::Bool(lb) => output.push(quote! {
                    eprintln!("SURPRISE: proc macro token parsing has changed now, true and false as parsed as literal booleans!");
                    #out_id.push_str(&#lb.to_string());
                }),
                Lit::Verbatim(v) => {
                    return Err(syn::Error::new(
                        current_span,
                        format!("unknown token literal {}, unable to convert to markup", v),
                    ));
                }
                _ => {
                    return Err(syn::Error::new(
                        current_span,
                        "unknown token type",
                    ));
                },
            },
            TokenTree::Group(g) => {
                match g.delimiter() {
                    Delimiter::Parenthesis => {
                        let inner = g.stream();
                        output.push(quote! {
                            #out_id.push_str(&lfml::Escapable::markup(&{#inner}).into_string());
                        });
                    },
                    Delimiter::Brace => {
                        process_tokens(g.stream().into_iter().peekable(), output, out_id)?;
                    },
                    Delimiter::Bracket => todo!(),
                    Delimiter::None => todo!(),
                }
            },
            TokenTree::Ident(i) => {
                if i == "true" || i == "false" {
                    let litbconv = Literal::string(&i.to_string());
                    output.push(quote! {
                        #out_id.push_str(#litbconv);
                    });

                    continue;
                }

                let opening_tag = &mut format!("<{i}");
                let closing_tag = &format!("</{i}>");
                let mut interp_attrs = vec![];

                'attrs: loop {
                    match tokens.peek() {
                        Some(TokenTree::Group(_)) => {
                            let TokenTree::Group(g) = tokens.next().unwrap() else { unreachable!() };
                            // println!("{g:?}");

                            match g.delimiter() {
                                Delimiter::Parenthesis => {
                                    let inner = g.stream();

                                    interp_attrs.push(quote! {{
                                        #inner
                                    }});

                                    opening_tag.push_str("\"{}\"");
                                },
                                Delimiter::Brace => {
                                    let opening_tag = Literal::string(&format!("{opening_tag}>"));
                                    let closing_tag = Literal::string(closing_tag);

                                    let push_opening_expr = if interp_attrs.is_empty() {
                                        quote! {
                                            #out_id.push_str(#opening_tag);
                                        }
                                    } else {
                                        quote! {
                                            #out_id.push_str(&format!(#opening_tag, #(#interp_attrs)*));
                                        }
                                    };

                                    output.push(push_opening_expr);

                                    process_tokens(g.stream().into_iter().peekable(), output, out_id)?;

                                    output.push(quote! {
                                        #out_id.push_str(#closing_tag);
                                    });

                                    continue 'main;
                                },
                                Delimiter::Bracket => todo!(),
                                Delimiter::None => todo!(),
                            }
                        },
                        Some(TokenTree::Punct(_)) => {
                            let TokenTree::Punct(p) = tokens.next().unwrap() else { unreachable!() };

                            match p.as_char() {
                                ';' => {
                                    let tag = Literal::string(&format!("{opening_tag}>"));
                                    output.push(quote! {
                                        #out_id.push_str(#tag);
                                    });
                                }
                                '=' => {
                                    opening_tag.push('=');

                                    continue 'attrs;
                                }
                                p => {
                                    todo!("{p}")
                                }
                            }

                            continue 'main;
                        },
                        Some(TokenTree::Ident(_)) => {
                            let TokenTree::Ident(i) = tokens.next().unwrap() else { unreachable!() };
                            if i == "true" || i == "false" {
                                opening_tag.push('"');
                                opening_tag.push_str(&i.to_string());
                                opening_tag.push('"');

                                continue 'attrs;
                            }
                            opening_tag.push(' ');
                            opening_tag.push_str(&i.to_string());
                        },
                        Some(TokenTree::Literal(_)) => {
                            let TokenTree::Literal(l) = tokens.next().unwrap() else { unreachable!() };
                            opening_tag.push('"');
                            match Lit::new(l) {
                                Lit::Str(s) => {
                                    opening_tag.push_str(&s.value());
                                },
                                Lit::ByteStr(bs) => {
                                    opening_tag.push_str(&String::from_utf8_lossy(&bs.value()));
                                },
                                Lit::Byte(b) => {
                                    opening_tag.push(b.value() as char);
                                },
                                Lit::Char(c) => {
                                    opening_tag.push(c.value());
                                },
                                Lit::Int(i) => {
                                    opening_tag.push_str(&i.to_string());
                                },
                                Lit::Float(f) => {
                                    opening_tag.push_str(&f.to_string());
                                },
                                Lit::Bool(b) => {
                                    opening_tag.push_str(&b.value().to_string());
                                },
                                _ => {
                                    return Err(syn::Error::new(
                                        current_span,
                                        "unknown token type",
                                    ));
                                }
                            }
                            opening_tag.push('"');
                            continue 'attrs;
                        }
                        None => {
                            return Err(syn::Error::new(current_span, format!("no rules expected ident \"{i}\" at the end of lfml")));
                        },
                    };
                }
            }
            TokenTree::Punct(p) => {
                match p.as_char() {
                    '-' => {
                        let Some(TokenTree::Literal(l)) = tokens.next() else {
                            return Err(syn::Error::new(current_span, "unexpected token -"));
                        };

                        match Lit::new(l) {
                            Lit::Int(x) => {
                                let litstr = Literal::string(&format!("-{}", x));
                                output.push(quote! {
                                    #out_id.push_str(&#litstr.to_string());
                                });
                            },
                            Lit::Float(x) => {
                                let litstr = Literal::string(&format!("-{}", x));
                                output.push(quote! {
                                    #out_id.push_str(&#litstr.to_string());
                                });
                            },
                            _ => todo!(),
                        }
                    }
                    ';' => {},
                    _ => {
                        panic!("{p} wasn't handled");
                    }
                }
            }
        }
    }
    Ok(())
}
