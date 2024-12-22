use crate::assert_html_eq;

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

#[test]
fn attrs_without_values_can_be_toggled_with_boolean_valued_expressions() {
    let x = true;

    assert_html_eq!({
        a foo[x] {
            "Hello"
        }
    } => "<a foo>Hello</a>");

    let x = false;

    assert_html_eq!({
        a foo[x] {
            "Hello"
        }
    } => "<a>Hello</a>");
}

#[test]
fn attrs_with_values_can_be_toggled_with_option_valued_expressions() {
    let x = Some(3);

    assert_html_eq!({
        a foo=[x] {
            "Hello"
        }
    } => "<a foo=\"3\">Hello</a>");

    let x: Option<i32> = None;

    assert_html_eq!({
        a foo=[x] {
            "Hello"
        }
    } => "<a>Hello</a>");
}

#[test]
fn attrs_can_be_literal_sequence() {
    let x = 3;

    assert_html_eq!({
        a foo={ "name_" (x) } {
            "Hello"
        }
    } => "<a foo=\"name_3\">Hello</a>");
}
