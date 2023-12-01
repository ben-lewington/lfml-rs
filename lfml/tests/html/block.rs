use crate::assert_html_eq;

use lfml::html;

#[test]
fn named_block() {
    assert_html_eq!({a {}} => "<a></a>");

    assert_html_eq!({
        a {
            "Hello"
        }
    } => "<a>Hello</a>");

    assert_html_eq!({
        a {
            "Hello"
            b {
                "World"
            }
        }
    } => "<a>Hello<b>World</b></a>");
}

#[test]
fn anon_block() {
    assert_html_eq!({{ "foo" "bar" }} => "foobar");

    assert_html_eq!({ "foo" { "bar" } "baz" } => "foobarbaz");
}

#[test]
fn self_closing_tag() {
    assert_html_eq!({ link; } => "<link>");
}
