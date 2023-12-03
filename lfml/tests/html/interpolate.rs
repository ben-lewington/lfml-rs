use crate::assert_html_eq;

use lfml::html;

#[test]
fn markup_body() {
    let x = "foobar";

    assert_html_eq!({
        (x)
    } => "foobar");

    let x = 3;

    assert_html_eq!({
        (x)
    } => "3");

    let x = html! {
        a {
            "Foo"
        }
    };

    assert_html_eq!({
        (x)
    } => "<a>Foo</a>");
}

#[test]
fn attrs() {
    let x = "bar";

    assert_html_eq!({
        a foo=(x) {
            "Hello"
        }
    } => "<a foo=\"bar\">Hello</a>");

    let x = 3;

    assert_html_eq!({
        a foo=(x) {
            "Hello"
        }
    } => "<a foo=\"3\">Hello</a>");
}

#[test]
fn attrs_content_is_escaped() {
    let x = "<danger>";

    assert_html_eq!({
        a foo=(x) {
            "Hello"
        }
    } => "<a foo=\"&lt;danger&gt;\">Hello</a>");
}
