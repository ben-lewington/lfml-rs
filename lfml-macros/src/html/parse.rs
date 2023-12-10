use crate::html::syntax::{
    Interpolate, InterpolateWrapper, Markup, MarkupAttr, MarkupId, MarkupLit,
};

use proc_macro2::{Delimiter, Ident, Literal, TokenStream, TokenTree};
use syn::Lit;

use super::{syntax::External, unnamed_tag_ident};

pub struct LfmlParser(pub proc_macro2::token_stream::IntoIter);

impl Iterator for LfmlParser {
    type Item = syn::Result<Markup>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.peek() {
                Some(TokenTree::Literal(_)) => {
                    let ls = match self.parse_literal_seq() {
                        Ok(ls) => ls,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                    return Some(Ok(Markup::LiteralSequence(ls)));
                }
                Some(TokenTree::Ident(i)) if i == "true" || i == "false" => {
                    let ls = match self.parse_literal_seq() {
                        Ok(ls) => ls,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                    return Some(Ok(Markup::LiteralSequence(ls)));
                }
                Some(TokenTree::Ident(_)) => {
                    let ident = match self.parse_ident() {
                        Ok(i) => i,
                        Err(e) => return Some(Err(e)),
                    };

                    let (attrs, inner) = match self.parse_attrs(ident.clone()) {
                        Ok(a) => a,
                        Err(e) => return Some(Err(e)),
                    };

                    let inner = if let Some(inner) = inner {
                        let mut v = vec![];
                        for t in LfmlParser(inner.into_iter()) {
                            let t = match t {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            };
                            v.push(t);
                        }
                        Some(v)
                    } else {
                        None
                    };

                    return Some(Ok(Markup::Tag {
                        tag: ident.clone(),
                        attrs,
                        inner,
                    }));
                }
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    self.advance();

                    let mut inner = vec![];
                    for t in LfmlParser(g.stream().into_iter()) {
                        match t {
                            Ok(t) => inner.push(t),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    return Some(Ok(Markup::AnonBlock(inner)));
                }
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                    self.advance();

                    return Some(Ok(Markup::Interpolated(External(g.stream()))));
                }
                Some(TokenTree::Punct(p)) => match p.as_char() {
                    '@' | '.' | '#' => {
                        let (attrs, inner) = match self.parse_attrs(unnamed_tag_ident()) {
                            Ok(a) => a,
                            Err(e) => return Some(Err(e)),
                        };

                        let inner = if let Some(inner) = inner {
                            let mut v = vec![];
                            for t in LfmlParser(inner.into_iter()) {
                                let t = match t {
                                    Ok(t) => t,
                                    Err(e) => return Some(Err(e)),
                                };
                                v.push(t);
                            }
                            Some(v)
                        } else {
                            None
                        };

                        return Some(Ok(Markup::Tag {
                            tag: unnamed_tag_ident(),
                            attrs,
                            inner,
                        }));
                    }
                    '-' => {
                        let ls = match self.parse_literal_seq() {
                            Ok(ls) => ls,
                            Err(e) => return Some(Err(e)),
                        };
                        return Some(Ok(Markup::LiteralSequence(ls)));
                    }
                    ';' => {
                        self.advance();
                    }
                    _ => todo!(),
                },
                Some(TokenTree::Group(g)) => {
                    return Some(Err(syn::Error::new(
                        g.span(),
                        "interpolating Option<impl Render> values ",
                    )));
                }
                None => return None,
            }
        }
    }
}

impl LfmlParser {
    fn peek(&self) -> Option<TokenTree> {
        self.0.clone().next()
    }

    fn advance(&mut self) -> Option<TokenTree> {
        self.0.next()
    }

    fn peek_2(&self) -> (Option<TokenTree>, Option<TokenTree>) {
        let mut ts = self.0.clone();
        let x0 = ts.next();
        let x1 = ts.next();
        (x0, x1)
    }

    fn advance_2(&mut self) {
        self.0.next();
        self.0.next();
    }

    fn parse_literal_seq(&mut self) -> syn::Result<Vec<MarkupLit>> {
        let mut inner = vec![];

        loop {
            match self.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == ';' => {
                    self.advance();
                }
                _ => {
                    let Some(l) = self.parse_literal()? else {
                        break;
                    };
                    inner.push(l);
                }
            }
        }
        Ok(inner)
    }

    fn parse_ident(&mut self) -> syn::Result<MarkupId> {
        let Some(TokenTree::Ident(i)) = self.advance() else {
            unreachable!()
        };

        let mut thr = String::new();
        match self.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                let mut expect_ident = true;
                loop {
                    expect_ident = match self.peek() {
                        Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                            self.advance();
                            thr.push('-');
                            true
                        }
                        Some(TokenTree::Ident(i)) if expect_ident => {
                            self.advance();
                            thr.push_str(&i.to_string());
                            false
                        }
                        _ => break,
                    };
                }
            }
            _ => {}
        };

        if thr.is_empty() {
            Ok(MarkupId::Basic(i))
        } else {
            Ok(MarkupId::Complex(i, thr))
        }
    }

    fn parse_literal(&mut self) -> syn::Result<Option<MarkupLit>> {
        match self.peek_2() {
            (Some(TokenTree::Literal(l)), _) => {
                self.advance();
                Ok(Some(MarkupLit::Basic(l)))
            }
            (Some(TokenTree::Punct(p)), Some(TokenTree::Literal(l))) if p.as_char() == '-' => {
                let s =
                    match Lit::new(l) {
                        Lit::Int(i) => i.base10_digits().to_string(),
                        Lit::Float(f) => f.base10_digits().to_string(),
                        _ => return Err(syn::Error::new(
                            p.span(),
                            "expected only integer or float literals after - as an attribute value",
                        )),
                    };

                self.advance_2();

                Ok(Some(MarkupLit::NegativeNumber(Literal::string(&s))))
            }
            (Some(TokenTree::Ident(i)), _) if i == "true" || i == "false" => {
                self.advance();
                Ok(Some(MarkupLit::Basic(Literal::string(&i.to_string()))))
            }
            _ => Ok(None),
        }
    }

    fn parse_attrs(
        &mut self,
        tag: MarkupId,
    ) -> syn::Result<(Vec<MarkupAttr>, Option<TokenStream>)> {
        let mut output = vec![];
        loop {
            match self.peek_2() {
                (Some(TokenTree::Punct(p)), _) if p.as_char() == ';' => {
                    self.advance();
                    return Ok((output, None));
                }
                (Some(TokenTree::Group(g)), _) if g.delimiter() == Delimiter::Brace => {
                    self.advance();
                    return Ok((output, Some(g.stream())));
                }
                (Some(TokenTree::Ident(_)), _) => {
                    let ident = self.parse_ident()?;
                    'attr: loop {
                        match self.peek_2() {
                            (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                                if p.as_char() == '='
                                    && g.delimiter() == Delimiter::Parenthesis =>
                            {
                                output.push(MarkupAttr::Interpolate {
                                    value: External(g.stream()),
                                    r#type: Interpolate::NameValue {
                                        name: ident.clone(),
                                        wrapper: InterpolateWrapper::None,
                                    },
                                });
                                self.advance_2();
                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                                if p.as_char() == '=' && g.delimiter() == Delimiter::Bracket =>
                            {
                                output.push(MarkupAttr::Interpolate {
                                    value: External(g.stream()),
                                    r#type: Interpolate::NameValue {
                                        name: ident.clone(),
                                        wrapper: InterpolateWrapper::Option,
                                    },
                                });
                                self.advance_2();
                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), _) if p.as_char() == '=' => {
                                self.advance();

                                let Some(l) = self.parse_literal()? else {
                                    return Err(syn::Error::new(
                                        p.span(),
                                        "Unable to parse literal",
                                    ));
                                };

                                output.push(MarkupAttr::Lit {
                                    name: ident.clone(),
                                    value: Some(l),
                                });

                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), _) if p.as_char() == ';' => {
                                self.advance();

                                output.push(MarkupAttr::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });

                                return Ok((output, None));
                            }
                            (Some(TokenTree::Group(g)), _)
                                if g.delimiter() == Delimiter::Bracket =>
                            {
                                output.push(MarkupAttr::Interpolate {
                                    value: External(g.stream()),
                                    r#type: Interpolate::Toggle {
                                        name: ident.clone(),
                                    },
                                });

                                self.advance();
                                break 'attr;
                            }
                            (Some(TokenTree::Group(g)), _) if g.delimiter() == Delimiter::Brace => {
                                output.push(MarkupAttr::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });
                                self.advance();
                                return Ok((output, Some(g.stream())));
                            }
                            (Some(TokenTree::Ident(_)), _) => {
                                output.push(MarkupAttr::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });
                                break 'attr;
                            }

                            _ => todo!(""),
                        }
                    }
                }
                (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                    if p.as_char() == '.' || p.as_char() == '#' || p.as_char() == '@' =>
                {
                    output.push(match (p.as_char(), g.delimiter()) {
                        ('.', Delimiter::Parenthesis) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::NameValue {
                                name: MarkupId::Basic(Ident::new("class", p.span())),
                                wrapper: InterpolateWrapper::None,
                            },
                        },
                        ('.', Delimiter::Bracket) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::NameValue {
                                name: MarkupId::Basic(Ident::new("class", p.span())),
                                wrapper: InterpolateWrapper::Option,
                            },
                        },
                        ('#', Delimiter::Parenthesis) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::NameValue {
                                name: MarkupId::Basic(Ident::new("id", p.span())),
                                wrapper: InterpolateWrapper::None,
                            },
                        },
                        ('#', Delimiter::Bracket) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::NameValue {
                                name: MarkupId::Basic(Ident::new("id", p.span())),
                                wrapper: InterpolateWrapper::Option,
                            },
                        },
                        ('@', Delimiter::Parenthesis) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::Spread {
                                tag: tag.clone(),
                                wrapper: InterpolateWrapper::None,
                            },
                        },
                        ('@', Delimiter::Bracket) => MarkupAttr::Interpolate {
                            value: External(g.stream()),
                            r#type: Interpolate::Spread {
                                tag: tag.clone(),
                                wrapper: InterpolateWrapper::Option,
                            },
                        },
                        _ => {
                            return Err(syn::Error::new(
                                p.span(),
                                format!("{p:?} {g:?} unhandled"),
                            ));
                        }
                    });
                    self.advance_2();
                }
                (Some(TokenTree::Punct(p)), Some(TokenTree::Ident(_)))
                    if p.as_char() == '.' || p.as_char() == '#' =>
                {
                    self.advance();

                    let i = self.parse_ident()?;

                    let attr_name = if p.as_char() == '.' { "class" } else { "id" };

                    output.push(MarkupAttr::Lit {
                        name: MarkupId::Basic(Ident::new(attr_name, p.span())),
                        value: Some(MarkupLit::Basic(Literal::string(&i.to_string()))),
                    });
                }
                (Some(TokenTree::Punct(p)), _) if p.as_char() == '.' || p.as_char() == '#' => {
                    self.advance();
                    let Some(l) = self.parse_literal()? else {
                        return Err(syn::Error::new(p.span(), "unable to parse literal"));
                    };
                    let attr_name = if p.as_char() == '.' { "class" } else { "id" };
                    output.push(MarkupAttr::Lit {
                        name: MarkupId::Basic(Ident::new(attr_name, p.span())),
                        value: Some(l),
                    })
                }
                _ => todo!("a"),
            }
        }
    }
}
