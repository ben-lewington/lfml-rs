use std::iter::Peekable;

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree, token_stream::IntoIter};
use quote::quote;
use syn::Lit;

const SIZE_MULTIPLIER: usize = 10;

pub fn generate_markup_expr(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let out_id = Ident::new("__lfml_output", Span::mixed_site());
    let size_hint = input.to_string().len() * SIZE_MULTIPLIER;

    let tokens = input.clone().into_iter().peekable();
    let mut output = vec![];

    process_tokens(tokens, &mut output, &out_id)?;

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
    while let Some(t) = tokens.next() {
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
            TokenTree::Group(_) => todo!("Group"),
            TokenTree::Ident(i) => {
                if i == "true" || i == "false" {
                    let litbconv = Literal::string(&i.to_string());
                    output.push(quote! {
                        #out_id.push_str(#litbconv);
                    });

                    continue;
                }

                if let Some(TokenTree::Group(_)) = tokens.peek() {
                    let token = tokens.next().unwrap();

                    let TokenTree::Group(g) = token else { unreachable!() };

                    let block_tokens = g.stream().into_iter().peekable();

                    let opening_tag = Literal::string(&format!("<{i}>"));
                    let closing_tag = Literal::string(&format!("</{i}>"));

                    output.push(quote! {
                        #out_id.push_str(#opening_tag);
                    });

                    process_tokens(block_tokens, output, out_id)?;

                    output.push(quote! {
                        #out_id.push_str(#closing_tag);
                    });
                }

                // todo!("everything else");
            }
            TokenTree::Punct(p) => {
                match p.to_string().as_str() {
                    "-" => {
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
                    _ => {
                        panic!("there");
                    }
                }
            }
        }
    }
    Ok(())
}
