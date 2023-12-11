use proc_macro2::Span;
use quote::ToTokens;
use syn::Lit;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Markup {
    LiteralSequence(Vec<MarkupLit>),
    Tag {
        tag: MarkupId,
        attrs: Vec<TagAttribute>,
        inner: Option<Vec<Markup>>,
    },
    AnonBlock(Vec<Markup>),
    Slot(External),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TagAttribute {
    Lit {
        name: MarkupId,
        value: Option<MarkupLit>,
    },
    Interpolated {
        r#type: InterpValue,
        value: External,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InterpValue {
    Toggle {
        name: MarkupId,
    },
    NameValue {
        name: MarkupId,
        wrapper: InterpValueType,
    },
    Spread {
        tag: MarkupId,
        wrapper: InterpValueType,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum InterpValueType {
    None,
    Option,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupId {
    Basic(proc_macro2::Ident),
    Complex(proc_macro2::Ident, String),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupLit {
    Basic(proc_macro2::Literal),
    NegativeNumber(proc_macro2::Literal),
}

#[derive(Debug, Clone)]
pub struct External(pub proc_macro2::TokenStream);

impl ToTokens for External {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl core::fmt::Display for MarkupId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkupId::Basic(b) => write!(f, "{}", b)?,
            MarkupId::Complex(b, c) => {
                write!(f, "{}", b)?;
                write!(f, "{}", c)?;
            }
        }
        Ok(())
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
