use crate::assert_html_eq;

use lfml::html;

#[test]
fn markup_body() {
    let x = "foobar";

    assert_html_eq!({
        (x)
    } => "foobar");

    let x = html! {
        a {
            "Foo"
        }
    };

    assert_html_eq!({
        (x)
    } => "<a>Foo</a>");
}
