use proc_macro2::Span;
use syn::Lit;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Markup {
    LiteralSequence(Vec<MarkupLit>),
    MarkupTag {
        ident: MarkupId,
        attrs: Vec<MarkupAttr>,
        inner: Option<Vec<Markup>>,
    },
    AnonBlock(Vec<Markup>),
    Interpolated(proc_macro2::TokenStream),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupAttr {
    Single {
        name: MarkupId,
    },
    Static {
        name: MarkupId,
        value: MarkupLit,
    },
    Interpolate {
        r#type: Interpolate,
        value: proc_macro2::TokenStream,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Interpolate {
    Toggle {
        name: MarkupId,
    },
    NameValue {
        name: MarkupId,
        wrapper: InterpolateWrapper,
    },
    Spread {
        tag: MarkupId,
        wrapper: InterpolateWrapper,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum InterpolateWrapper {
    None,
    Option,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupId {
    Unnamed,
    Basic(proc_macro2::Ident),
    Complex(String),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupLit {
    Basic(proc_macro2::Literal),
    NegativeNumber(proc_macro2::Literal),
}

impl core::fmt::Display for MarkupId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MarkupId::Unnamed => crate::html::UNNAMED_TAG.into(),
                MarkupId::Basic(b) => b.to_string(),
                MarkupId::Complex(c) => c.to_owned(),
            }
        )
    }
}

impl MarkupLit {
    pub fn span(&self) -> Span {
        match self {
            MarkupLit::Basic(b) => b.span(),
            MarkupLit::NegativeNumber(b) => b.span(),
        }
    }

    pub fn push_to_string(&self, buf: &mut String) -> syn::Result<()> {
        match self {
            MarkupLit::Basic(l) => {
                match Lit::new(l.clone()) {
                    Lit::Str(s) => {
                        lfml_escape::escape_to_string(&s.value(), buf);
                    }
                    Lit::ByteStr(bs) => {
                        lfml_escape::escape_to_string(&String::from_utf8_lossy(&bs.value()), buf);
                    }
                    Lit::Byte(b) => {
                        lfml_escape::escape_to_string(&String::from(b.value() as char), buf);
                    }
                    Lit::Char(c) => {
                        lfml_escape::escape_to_string(&String::from(c.value()), buf);
                    }
                    Lit::Int(i) => {
                        let i = i.base10_digits().parse::<usize>().expect("parsing litint");
                        buf.push_str(&i.to_string());
                    }
                    Lit::Float(lf) => {
                        let lf = lf.base10_digits().parse::<f64>().expect("parsing float");
                        buf.push_str(&lf.to_string());
                    }
                    Lit::Bool(lb) => {
                        eprintln!("SURPRISE: proc macro token parsing has changed now, true and false as parsed as literal booleans!");
                        buf.push_str(&lb.value.to_string());
                    }
                    Lit::Verbatim(v) => {
                        return Err(syn::Error::new(
                            self.span(),
                            format!("unknown token literal {}, unable to convert to markup", v),
                        ));
                    }
                    _ => {
                        return Err(syn::Error::new(self.span(), "unknown token type"));
                    }
                };
            }
            MarkupLit::NegativeNumber(l) => match Lit::new(l.clone()) {
                Lit::Str(s) => {
                    buf.push_str(&format!("-{}", s.value()));
                }
                _ => {
                    unreachable!()
                }
            },
        };
        Ok(())
    }
}
