use proc_macro2::Span;
use quote::ToTokens;
use syn::Lit;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Markup {
    /// list of parsed literals (possibly `;` punctuated)
    /// ```"abc" 123 true -100```
    LiteralSequence(Vec<MarkupLit>),
    /// parsed markup_block
    /// ```#tag #(#attrs)* { #inner }```
    Tag {
        tag: MarkupId,
        attrs: Vec<TagAttribute>,
        inner: Option<Vec<Markup>>,
    },
    /// anonymous block
    /// { #inner }
    AnonBlock(Vec<Markup>),
    /// lfml::Markup-valued rust expression that is interpolated into the rendered template.
    Slot(InterpMarkupExpr),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InterpMarkupExpr {
    /// (#expr)
    Simple(External),
    /// ```ignore
    /// @#match_expr {
    ///     #(#variant => { #markup_expr },)*
    /// }
    /// ```
    Match(External, Vec<(External, Vec<Markup>)>),
    /// ```ignore
    /// @#if_expr {
    ///     #markup_expr
    /// } #(@#else_expr {
    ///     #markup_expr
    /// })*
    /// ```
    If {
        if_block: (External, Vec<Markup>),
        else_blocks: Vec<(External, Vec<Markup>)>,
    },
    /// ```ignore
    /// @#for_expr {
    ///     #markup_expr
    /// }
    /// ```
    For(External, Vec<Markup>),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TagAttribute {
    /// An attribute literal name-value, e.g. #name="value", or without a value, e.g. #name
    Lit {
        name: MarkupId,
        value: Option<MarkupLit>,
    },
    /// Attribute values which are to be interpolated
    Interpolated {
        r#type: InterpValue,
        value: External,
    },
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InterpValue {
    /// ```ignore
    /// #name[#bool_expr]
    /// ```
    Toggle { name: MarkupId },
    /// ```ignore
    /// #name=(#expr)
    /// // or
    /// #name=[#expr]
    /// ```
    NameValue {
        name: MarkupId,
        wrapper: InterpValueType,
    },
    /// ```ignore
    /// #tag @(#spread_expr)
    /// // or
    /// #tag @[#option_spread_expr]
    /// ```
    Spread {
        tag: MarkupId,
        wrapper: InterpValueType,
    },
}

/// When interpolating an expression into a markup expression, we require the resultant expression
/// implements Render (Display in attribute position).
///
/// This represents the wrapper type that the expression is expected to evaluate to. This way the
/// generated code can treat the inner value as an Option<Markup>.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum InterpValueType {
    /// Inner type is expected to be impl Render
    None,
    /// Inner type is expected to be Option<impl Render/Display>
    Option,
}

/// Identifiers in html can be either a rust identifier, or a hyphen separated list of identifiers
/// (that starts and ends with an identifier).
/// e.g.
/// ```ignore
/// #ident
/// //or
/// #(#ident -)*#ident
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupId {
    Basic(proc_macro2::Ident),
    Complex(proc_macro2::Ident, String),
}

/// A literal value to be rendered into the final markup.
/// it will either be a rust literal, or a -ve number
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MarkupLit {
    Basic(proc_macro2::Literal),
    NegativeNumber(proc_macro2::Literal),
}

/// A TokenStream which is supposed to be parsed as rust code, rather than markup.
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
                write!(f, "{}{}", b, c)?;
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
                        eprintln!("true and false are parsed as identifiers, not literal booleans!");
                        buf.push_str(&lb.value.to_string());
                    }
                    Lit::Verbatim(v) => {
                        return Err(syn::Error::new(
                            self.span(),
                            format!("unknown token literal {v}, unable to convert to markup"),
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
                },
                _ => unreachable!(),
            },
        };
        Ok(())
    }
}
