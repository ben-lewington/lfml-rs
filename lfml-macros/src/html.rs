use proc_macro2::{Ident, Literal, Span, TokenTree};
use quote::quote;
use syn::Lit;

pub fn generate_markup_expr(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let out_id = Ident::new("__lfml_output", Span::mixed_site());
    let size_hint = input.to_string().len();

    let mut tokens = input.clone().into_iter().peekable();
    let mut datum = vec![];

    while let Some(t) = tokens.next() {
        let current_span = t.span();
        match t {
            TokenTree::Literal(l) => match Lit::new(l) {
                Lit::Str(s) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&s.value()));
                    datum.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::ByteStr(bs) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(
                        &String::from_utf8_lossy(&bs.value()),
                    ));
                    datum.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Byte(b) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&String::from(
                        b.value() as char,
                    )));
                    datum.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Char(c) => {
                    let litstr = Literal::string(&lfml_escape::escape_string(&String::from(
                        c.value()
                    )));
                    datum.push(quote! {
                        #out_id.push_str(#litstr);
                    });
                }
                Lit::Int(i) => datum.push(quote! {
                    #out_id.push_str(&#i.to_string());
                }),
                Lit::Float(lf) => datum.push(quote! {
                    #out_id.push_str(&#lf.to_string());
                }),
                Lit::Bool(lb) => datum.push(quote! {
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
            TokenTree::Group(_) => todo!(),
            TokenTree::Ident(i) => {
                if i == "true" || i == "false" {
                    println!("here");
                    let litbconv = Literal::string(&i.to_string());
                    datum.push(quote! {
                        #out_id.push_str(#litbconv);
                    })
                }
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
                                datum.push(quote! {
                                    #out_id.push_str(&#litstr.to_string());
                                });
                            },
                            Lit::Float(x) => {
                                let litstr = Literal::string(&format!("-{}", x));
                                datum.push(quote! {
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

    Ok(quote! {{
        let mut #out_id = String::with_capacity(#size_hint);
        #(#datum)*
        lfml::Escaped(#out_id)
    }})
}
