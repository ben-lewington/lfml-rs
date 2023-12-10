use lfml::{Render, html};

#[derive(lfml::MarkupAttrs)]
#[tags(link)]
struct Link<'a> {
    href: &'a str,
    rel: LinkRel,
    r#as: Option<LinkAs>,
}

impl<'a> Link<'a> {
    fn new(href: &'a str, rel: LinkRel, r#as: Option<LinkAs>) -> Self {
        Self { href, rel, r#as }
    }
}

#[non_exhaustive]
enum LinkRel {
    StyleSheet,
    PreLoad,
}

#[non_exhaustive]
enum LinkAs {
    Image,
}


impl core::fmt::Display for LinkAs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            LinkAs::Image => "image",
            // _ => todo!()
        })
    }
}

impl core::fmt::Display for LinkRel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            LinkRel::StyleSheet => "stylesheet",
            LinkRel::PreLoad => "preload",
        })
    }
}

#[derive(rust_embed::RustEmbed)]
#[folder = "static"]
struct StaticFiles;

enum Hypermedia<M> {
    Document(M),
    Fragment(M),
}

impl <M: Render> Render for Hypermedia<M> {
    fn markup(&self) -> lfml::Escaped<String> {
        match self {
            Hypermedia::Document(d) => {

            },
            Hypermedia::Fragment(f) => todo!(),
        }
        html!  { "" }

    }
}
