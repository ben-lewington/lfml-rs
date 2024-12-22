use crate::html::syntax::{
    InterpMarkupExpr, InterpValue, InterpValueType, Markup, MarkupId, MarkupLit, TagAttribute,
};

use proc_macro2::{Delimiter, Ident, Literal, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Lit;

use super::{syntax::External, unnamed_tag_ident};

pub struct LfmlParser(pub proc_macro2::token_stream::IntoIter);

impl Iterator for LfmlParser {
    type Item = syn::Result<Markup>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! result_to_option {
            ($($expr:tt)*) => {
                match { $($expr)* } {
                    Ok(a) => a,
                    Err(e) => return Some(Err(e)),
                }
            };
        }
        loop {
            match self.peek() {
                Some(TokenTree::Literal(_)) => {
                    let ls = result_to_option!(self.parse_literal_seq());

                    return Some(Ok(Markup::LiteralSequence(ls)));
                }
                Some(TokenTree::Ident(i)) if i == "true" || i == "false" => {
                    let ls = result_to_option!(self.parse_literal_seq());

                    return Some(Ok(Markup::LiteralSequence(ls)));
                }
                Some(TokenTree::Ident(_)) => {
                    let ident = result_to_option!(self.parse_ident());

                    let (attrs, inner) = result_to_option!(self.parse_attrs(ident.clone()));

                    let inner = if let Some(inner) = inner {
                        let mut v = vec![];
                        for t in LfmlParser(inner.into_iter()) {
                            v.push(result_to_option!(t));
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
                        inner.push(result_to_option!(t));
                    }
                    return Some(Ok(Markup::AnonBlock(inner)));
                }
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                    self.advance();

                    return Some(Ok(Markup::Slot(InterpMarkupExpr::Simple(External(
                        g.stream(),
                    )))));
                }
                Some(TokenTree::Punct(p)) => match p.as_char() {
                    '@' | '.' | '#' => {
                        match self.peek_2() {
                            (_, Some(TokenTree::Ident(i)))
                                if p.as_char() == '@'
                                    && (i == "match" || i == "if" || i == "for") =>
                            {
                                self.advance();
                                if i == "match" {
                                    let expr = result_to_option!(self.parse_match());

                                    return Some(Ok(Markup::Slot(expr)));
                                } else if i == "for" {
                                    let expr = result_to_option!(self.parse_for());

                                    return Some(Ok(Markup::Slot(expr)));
                                } else if i == "if" {
                                    let expr = result_to_option!(self.parse_if());

                                    return Some(Ok(Markup::Slot(expr)));
                                }
                            }
                            _ => {
                                // todo!("{t:?}");
                            }
                        }
                        let tag = unnamed_tag_ident();

                        let (attrs, inner) = result_to_option!(self.parse_attrs(tag.clone()));

                        let inner = if let Some(inner) = inner {
                            Some(result_to_option!(
                                LfmlParser(inner.into_iter()).collect::<syn::Result<Vec<_>>>()
                            ))
                        } else {
                            None
                        };

                        return Some(Ok(Markup::Tag { tag, attrs, inner }));
                    }
                    '-' => {
                        let ls = result_to_option!(self.parse_literal_seq());

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

    fn peek_3(&self) -> (Option<TokenTree>, Option<TokenTree>, Option<TokenTree>) {
        let mut ts = self.0.clone();
        let x0 = ts.next();
        let x1 = ts.next();
        let x2 = ts.next();
        (x0, x1, x2)
    }

    fn advance_2(&mut self) {
        self.0.next();
        self.0.next();
    }

    fn advance_3(&mut self) {
        self.0.next();
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

    fn parse_match(&mut self) -> syn::Result<InterpMarkupExpr> {
        let match_kw = match self.advance() {
            Some(TokenTree::Ident(i)) if i == "match" => i,
            t => {
                return Err(syn::Error::new(
                    t.map(|t| t.span()).unwrap_or(Span::mixed_site()),
                    "expected `match` ident",
                ))
            }
        };
        let mut outer_ext = match_kw.to_token_stream();
        let mut variants = vec![];
        'outer: loop {
            match self.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    self.advance();
                    let mut s = Self(g.stream().into_iter());

                    let mut inner_ext = TokenStream::new();
                    'variants: loop {
                        match s.peek_3() {
                            (
                                Some(TokenTree::Punct(eq)),
                                Some(TokenTree::Punct(rb)),
                                Some(TokenTree::Group(g)),
                            ) if (eq.as_char() == '=')
                                && rb.as_char() == '>'
                                && g.delimiter() == Delimiter::Brace =>
                            {
                                s.advance_3();
                                let ms: Vec<Markup> = Self(g.stream().into_iter())
                                    .collect::<syn::Result<Vec<_>>>()?;

                                variants.push((External(inner_ext.clone()), ms));
                                inner_ext = TokenStream::new();

                                continue 'variants;
                            }
                            (Some(t), _, _) => {
                                t.to_tokens(&mut inner_ext);
                                s.advance();
                            }
                            (None, _, _) => break 'variants,
                        };
                    }

                    break 'outer;
                }
                Some(t) => {
                    t.to_tokens(&mut outer_ext);
                    self.advance();
                }
                None => return Err(syn::Error::new(match_kw.span(), "unexpected end of macro")),
            }
        }
        Ok(InterpMarkupExpr::Match(External(outer_ext), variants))
    }

    fn parse_if(&mut self) -> syn::Result<InterpMarkupExpr> {
        let if_kw = match self.advance() {
            Some(TokenTree::Ident(i)) if i == "if" => i,
            t => {
                return Err(syn::Error::new(
                    t.map(|t| t.span()).unwrap_or(Span::mixed_site()),
                    "expected `if` ident",
                ))
            }
        };

        let mut outer_ext = if_kw.to_token_stream();
        let outer_markup: Vec<Markup>;
        let mut else_blocks = vec![];
        'outer: loop {
            match self.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    self.advance();
                    outer_markup = Self(g.stream().into_iter()).collect::<syn::Result<Vec<_>>>()?;

                    'test_if: loop {
                        match self.peek_2() {
                            (Some(TokenTree::Punct(p)), Some(TokenTree::Ident(else_kw)))
                                if p.as_char() == '@' && else_kw == "else" =>
                            {
                                let mut else_ext = else_kw.to_token_stream();
                                self.advance_2();
                                loop {
                                    match self.peek() {
                                        Some(TokenTree::Group(g))
                                            if g.delimiter() == Delimiter::Brace =>
                                        {
                                            self.advance();
                                            let else_markup = Self(g.stream().into_iter())
                                                .collect::<syn::Result<Vec<_>>>()?;

                                            else_blocks
                                                .push((External(else_ext.clone()), else_markup));

                                            continue 'test_if;
                                        }
                                        Some(t) => {
                                            t.to_tokens(&mut else_ext);
                                            self.advance();
                                        }
                                        None => {
                                            return Err(syn::Error::new(
                                                if_kw.span(),
                                                "unexpected end of macro",
                                            ))
                                        }
                                    }
                                }
                            }
                            _ => break 'outer,
                        }
                    }
                }
                Some(t) => {
                    t.to_tokens(&mut outer_ext);
                    self.advance();
                }
                None => return Err(syn::Error::new(if_kw.span(), "unexpected end of macro")),
            }
        }
        Ok(InterpMarkupExpr::If {
            if_block: (External(outer_ext), outer_markup),
            else_blocks,
        })
    }

    fn parse_for(&mut self) -> syn::Result<InterpMarkupExpr> {
        let for_kw = match self.advance() {
            Some(TokenTree::Ident(i)) if i == "for" => i,
            t => {
                return Err(syn::Error::new(
                    t.map(|t| t.span()).unwrap_or(Span::mixed_site()),
                    "expected `for` ident",
                ))
            }
        };
        let mut outer_ext = for_kw.to_token_stream();
        let mut in_kw = None;
        let repeating_blocks;
        loop {
            match self.peek() {
                Some(TokenTree::Group(g))
                    if g.delimiter() == Delimiter::Brace && in_kw.is_some() =>
                {
                    self.advance();

                    repeating_blocks = match Self(g.stream().into_iter()).collect() {
                        Ok(ms) => ms,
                        Err(e) => return Err(e),
                    };

                    break;
                }
                Some(TokenTree::Ident(i)) if i == "in" => {
                    in_kw.replace(i.clone());
                    i.to_tokens(&mut outer_ext);
                    self.advance();
                }
                Some(t) => {
                    t.to_tokens(&mut outer_ext);
                    self.advance();
                }
                None => return Err(syn::Error::new(for_kw.span(), "unexpected end of macro")),
            }
        }
        Ok(InterpMarkupExpr::For(External(outer_ext), repeating_blocks))
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
    ) -> syn::Result<(Vec<TagAttribute>, Option<TokenStream>)> {
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
                                output.push(TagAttribute::Interpolated {
                                    value: External(g.stream()),
                                    r#type: InterpValue::NameValue {
                                        name: ident.clone(),
                                        wrapper: InterpValueType::None,
                                    },
                                });
                                self.advance_2();
                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                                if p.as_char() == '=' && g.delimiter() == Delimiter::Bracket =>
                            {
                                output.push(TagAttribute::Interpolated {
                                    value: External(g.stream()),
                                    r#type: InterpValue::NameValue {
                                        name: ident.clone(),
                                        wrapper: InterpValueType::Option,
                                    },
                                });
                                self.advance_2();
                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), _) if p.as_char() == '=' => {
                                self.advance();

                                match self.peek() {
                                    Some(TokenTree::Literal(_)) | Some(TokenTree::Ident(_)) => {}
                                    Some(TokenTree::Group(g)) => {
                                        let l: Vec<Markup> = LfmlParser(g.stream().into_iter())
                                            .collect::<Result<Vec<Markup>, syn::Error>>()?;

                                        output.push(TagAttribute::Block {
                                            name: ident.clone(),
                                            value: l,
                                        });
                                        self.advance();
                                        break 'attr;
                                    }
                                    Some(TokenTree::Punct(_)) => {}
                                    None => {
                                        return Err(syn::Error::new(
                                            p.span(),
                                            "unexpected end of token stream",
                                        ))
                                    }
                                }
                                let Some(l) = self.parse_literal()? else {
                                    return Err(syn::Error::new(
                                        p.span(),
                                        "Unable to parse literal",
                                    ));
                                };

                                output.push(TagAttribute::Lit {
                                    name: ident.clone(),
                                    value: Some(l),
                                });

                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), _) if p.as_char() == ';' => {
                                self.advance();

                                output.push(TagAttribute::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });

                                return Ok((output, None));
                            }
                            (Some(TokenTree::Group(g)), _)
                                if g.delimiter() == Delimiter::Bracket =>
                            {
                                output.push(TagAttribute::Interpolated {
                                    value: External(g.stream()),
                                    r#type: InterpValue::Toggle {
                                        name: ident.clone(),
                                    },
                                });

                                self.advance();
                                break 'attr;
                            }
                            (Some(TokenTree::Group(g)), _) if g.delimiter() == Delimiter::Brace => {
                                output.push(TagAttribute::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });
                                self.advance();
                                return Ok((output, Some(g.stream())));
                            }
                            (Some(TokenTree::Ident(_)), _) => {
                                output.push(TagAttribute::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });
                                break 'attr;
                            }
                            (Some(TokenTree::Punct(p)), _)
                                if p.as_char() == '#'
                                    || p.as_char() == '.'
                                    || p.as_char() == '@' =>
                            {
                                output.push(TagAttribute::Lit {
                                    name: ident.clone(),
                                    value: None,
                                });
                                break 'attr;
                            }
                            t => todo!("unexpected token whilst parsing attr {t:?}"),
                        }
                    }
                }
                (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                    if p.as_char() == '.' || p.as_char() == '#' || p.as_char() == '@' =>
                {
                    output.push(match (p.as_char(), g.delimiter()) {
                        ('.', Delimiter::Parenthesis) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::NameValue {
                                name: MarkupId::Basic(Ident::new("class", p.span())),
                                wrapper: InterpValueType::None,
                            },
                        },
                        ('.', Delimiter::Bracket) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::NameValue {
                                name: MarkupId::Basic(Ident::new("class", p.span())),
                                wrapper: InterpValueType::Option,
                            },
                        },
                        ('#', Delimiter::Parenthesis) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::NameValue {
                                name: MarkupId::Basic(Ident::new("id", p.span())),
                                wrapper: InterpValueType::None,
                            },
                        },
                        ('#', Delimiter::Bracket) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::NameValue {
                                name: MarkupId::Basic(Ident::new("id", p.span())),
                                wrapper: InterpValueType::Option,
                            },
                        },
                        ('@', Delimiter::Parenthesis) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::Spread {
                                tag: tag.clone(),
                                wrapper: InterpValueType::None,
                            },
                        },
                        ('@', Delimiter::Bracket) => TagAttribute::Interpolated {
                            value: External(g.stream()),
                            r#type: InterpValue::Spread {
                                tag: tag.clone(),
                                wrapper: InterpValueType::Option,
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

                    output.push(TagAttribute::Lit {
                        name: MarkupId::Basic(Ident::new(attr_name, p.span())),
                        value: Some(MarkupLit::Basic(Literal::string(&i.to_string()))),
                    });
                }
                (Some(TokenTree::Punct(p)), _) if p.as_char() == '.' || p.as_char() == '#' => {
                    self.advance();
                    let Some(l) = self.parse_literal()? else {
                        return Err(syn::Error::new(
                            p.span(),
                            format!("unable to parse literal {}", p.as_char()),
                        ));
                    };
                    let attr_name = if p.as_char() == '.' { "class" } else { "id" };
                    output.push(TagAttribute::Lit {
                        name: MarkupId::Basic(Ident::new(attr_name, p.span())),
                        value: Some(l),
                    })
                }
                t => todo!("{t:?}"),
            }
        }
    }
}
