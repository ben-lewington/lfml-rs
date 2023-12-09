use proc_macro2::Span;
use syn::Lit;

#[derive(Debug, Clone)]
pub enum MarkupSyntax {
    LiteralSequence(Vec<MarkupLiteral>),
    MarkupTag {
        ident: MarkupIdent,
        attrs: Vec<MarkupAttrSyntax>,
        inner: Option<Vec<MarkupSyntax>>,
    },
    AnonBlock(Vec<MarkupSyntax>),
    Interpolated(proc_macro2::TokenStream),
}

#[derive(Debug, Clone)]
pub enum MarkupAttrSyntax {
    Single {
        name: MarkupIdent,
    },
    Static {
        name: MarkupIdent,
        value: MarkupLiteral,
    },
    Interpolate {
        value: proc_macro2::TokenStream,
        r#type: Interp,
    },
}

#[derive(Debug, Clone)]
pub enum Interp {
    Toggle {
        name: MarkupIdent,
    },
    KeyValue {
        name: MarkupIdent,
        unwrap: InterpUnwrap,
    },
    Spread {
        tag: MarkupIdent,
        unwrap: InterpUnwrap,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum InterpUnwrap {
    None,
    Option,
}

#[derive(Debug, Clone)]
pub enum MarkupIdent {
    Unnamed,
    Basic(proc_macro2::Ident),
    Complex(String),
}

#[derive(Debug, Clone)]
pub enum MarkupLiteral {
    Basic(proc_macro2::Literal),
    NegativeNumber(proc_macro2::Literal),
}

impl core::fmt::Display for MarkupIdent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MarkupIdent::Unnamed => crate::html::UNNAMED_TAG.into(),
                MarkupIdent::Basic(b) => b.to_string(),
                MarkupIdent::Complex(c) => c.to_owned(),
            }
        )
    }
}

impl MarkupLiteral {
    pub fn span(&self) -> Span {
        match self {
            MarkupLiteral::Basic(b) => b.span(),
            MarkupLiteral::NegativeNumber(b) => b.span(),
        }
    }

    pub fn push_to_string(&self, buf: &mut String) -> syn::Result<()> {
        match self {
            MarkupLiteral::Basic(l) => {
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
            MarkupLiteral::NegativeNumber(l) => match Lit::new(l.clone()) {
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
