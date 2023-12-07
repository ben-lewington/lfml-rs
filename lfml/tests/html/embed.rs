use crate::assert_html_eq;

use lfml::{html, EmbedAsAttrs};

#[test]
fn do_the_thing() {
    #[derive(EmbedAsAttrs)]
    #[prefix = "hx"]
    struct HxGet<'a> {
        get: &'a str,
        target: &'a str,
        swap: &'a str,
    }

    let x = HxGet {
        get: "/a",
        target: ".main",
        swap: "outerHTML",
    };

    assert_html_eq!({
        a@x { "A" }
    } => "<a hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\" >A</a>");

    assert_html_eq!({
        a@{
            HxGet {
                get: "/a",
                target: ".main",
                swap: "outerHTML",
            }
        } {
            "A"
        }
    } => "<a hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\" >A</a>")
}
