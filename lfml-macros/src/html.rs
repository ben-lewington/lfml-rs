use std::iter::Peekable;

use proc_macro2::{
    token_stream::IntoIter, Delimiter, Ident, Literal, Span, TokenStream, TokenTree,
};
use quote::{format_ident, quote};
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
                            #out_id.push_str(&lfml::Render::markup(&{#inner}).as_string());
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
                let closing_tag = &mut format!("</{i}");

                if let Some(TokenTree::Punct(p)) = tokens.peek() {
                    if p.as_char() == '-' {
                        let mut expect_ident = true;
                        loop {
                            expect_ident = match tokens.peek() {
                                Some(TokenTree::Punct(ref punct)) if punct.as_char() == '-' => {
                                    let TokenTree::Punct(_) = tokens.next().unwrap() else { unreachable!() };
                                    true
                                }
                                Some(TokenTree::Ident(_)) if expect_ident => {
                                    let TokenTree::Ident(ident) = tokens.next().unwrap() else { unreachable!() };
                                    opening_tag.push_str(&format!("-{ident}"));
                                    closing_tag.push_str(&format!("-{ident}"));
                                    false
                                }
                                _ => break,
                            };
                        }
                    }
                }

                closing_tag.push('>');
                let mut interp_attrs = vec![];

                'attrs: loop {
                    match tokens.peek() {
                        Some(TokenTree::Group(_)) => {
                            let TokenTree::Group(g) = tokens.next().unwrap() else { unreachable!() };

                            match g.delimiter() {
                                Delimiter::Parenthesis => {
                                    let inner = g.stream();

                                    interp_attrs.push(quote! {
                                        lfml::escape_string(&{
                                            #inner
                                        }.to_string())
                                    });

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
                                            #out_id.push_str(&format!(#opening_tag, #(#interp_attrs),*));
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
                                '@' => {
                                    match tokens.next() {
                                        // Some(TokenTree::Group(g)) => todo!(),
                                        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                                            let inner = g.stream();
                                            let impl_id = format_ident!("__lfml_tag_{i}");

                                            interp_attrs.push(quote! { {
                                                if let Some(inner) = {&#inner} {
                                                    inner.#impl_id()
                                                } else {
                                                    "".into()
                                                }
                                            }});

                                            opening_tag.push_str("{}");

                                            continue 'attrs;
                                        }
                                        Some(token @ TokenTree::Ident(_)) | Some(token @ TokenTree::Group(_)) => {
                                            let impl_id = format_ident!("__lfml_tag_{i}");

                                            interp_attrs.push(quote! { {
                                                {&#token}.#impl_id()
                                            }});

                                            opening_tag.push_str("{}");

                                            continue 'attrs;
                                        }
                                         _ => return Err(syn::Error::new(p.span(), "unsupported use of @"))
                                    }

                                }
                                ';' => {
                                    let tag = Literal::string(&format!("{opening_tag}>"));

                                    output.push(quote! {
                                        #out_id.push_str(#tag);
                                    });

                                    continue 'main;
                                },
                                p => {
                                    todo!("{p}")
                                }
                            }
                        },
                        Some(TokenTree::Ident(_)) => {
                            let TokenTree::Ident(i) = tokens.next().unwrap() else { unreachable!() };
                            if i == "true" || i == "false" {
                                opening_tag.push('"');
                                opening_tag.push_str(&i.to_string());
                                opening_tag.push('"');

                                continue 'attrs;
                            }

                            match tokens.peek() {
                                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                                    let Some(TokenTree::Group(g)) = tokens.next() else { unreachable!() };


                                    let litstr = Literal::string(&format!(" {i}"));

                                    let inner = g.stream();

                                    interp_attrs.push(quote! {
                                        if { #inner } {
                                            #litstr
                                        } else {
                                            ""
                                        }
                                    });

                                    opening_tag.push_str("{}");

                                    continue 'attrs;
                                }
                                Some(TokenTree::Punct(p)) if p.as_char() == ';' => {
                                    opening_tag.push(' ');
                                    opening_tag.push_str(&i.to_string());

                                    let tag = Literal::string(&format!("{opening_tag}>"));

                                    output.push(quote! {
                                        #out_id.push_str(#tag);
                                    });

                                    continue 'main;
                                },
                                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                                    let Some(TokenTree::Punct(_)) = tokens.next() else { unreachable!() };

                                    if let Some(TokenTree::Group(g)) = tokens.peek() {
                                        if let Delimiter::Bracket = g.delimiter() {
                                            let Some(TokenTree::Group(g)) = tokens.next() else { unreachable!() };

                                            let inner = g.stream();

                                            let litstr = Literal::string(&format!(" {}=\"{{}}\"", i));

                                            interp_attrs.push(quote! {
                                                if let Some(e) = { #inner } {
                                                    format!(#litstr, e)
                                                } else {
                                                    "".into()
                                                }
                                            });

                                            opening_tag.push_str("{}");

                                            continue 'attrs;
                                        } else {
                                            opening_tag.push(' ');
                                            opening_tag.push_str(&i.to_string());
                                            opening_tag.push('=');

                                            continue 'attrs;
                                        }
                                    } else {
                                        opening_tag.push(' ');
                                        opening_tag.push_str(&i.to_string());
                                        opening_tag.push('=');

                                        continue 'attrs;
                                    }
                                },
                                _ => {
                                        opening_tag.push(' ');
                                        opening_tag.push_str(&i.to_string());
                                        continue 'attrs;
                                }
                            }
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

pub(crate) static VALID_HTML5_TAGS: &[&str] = &[
    "a",
    "abbr",
    "address",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "bdi",
    "bdo",
    "blink",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "map",
    "mark",
    "marquee",
    "meta",
    "meter",
    "nav",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "pre",
    "progress",
    "q",
    "ruby",
    "s",
    "samp",
    "script",
    "section",
    "select",
    "small",
    "source",
    "span",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "ul",
    "var",
    "video",
    "wbr",
];
