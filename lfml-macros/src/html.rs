mod generate;
mod parse;
mod syntax;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use self::{generate::markup_as_string_push_operations, parse::LfmlParser, syntax::MarkupId};

const SIZE_MULTIPLIER: usize = 5;
const OUT_ID: &str = "__lfml_output";
const UNNAMED_TAG: &str = "div";

fn output_ident() -> Ident {
    Ident::new(OUT_ID, Span::mixed_site())
}

fn unnamed_tag_ident() -> MarkupId {
    MarkupId::Basic(Ident::new(UNNAMED_TAG, Span::mixed_site()))
}

pub fn generate_markup_expr(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let out_id = output_ident();
    let size_hint = input.to_string().len() * SIZE_MULTIPLIER;

    let mut ast = vec![];
    for s in LfmlParser(input.into_iter()) {
        ast.push(s?);
    }

    let mut output = TokenStream::new();

    markup_as_string_push_operations(&output_ident(), ast, &mut output)?;

    Ok(quote! {{
        let mut #out_id = String::with_capacity(#size_hint);
        #output
        lfml::Escaped(#out_id)
    }})
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
