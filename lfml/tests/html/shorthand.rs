use crate::assert_html_eq;

#[test]
fn class_shorthand() {
    assert_html_eq!({
        a ."foo" {
            "bar"
        }
    } => "<a class=\"foo\">bar</a>");

    assert_html_eq!({
        a .foo {
            "bar"
        }
    } => "<a class=\"foo\">bar</a>");

    assert_html_eq!({
        a .(3) {
            "bar"
        }
    } => "<a class=\"3\">bar</a>");
}

#[test]
fn class_toggle() {
    assert_html_eq!({
        a .[Some(3)] {
            "bar"
        }
    } => "<a class=\"3\">bar</a>");

    assert_html_eq!({
        a .[Option::<i32>::None] {
            "bar"
        }
    } => "<a>bar</a>");
}

#[test]
fn id_shorthand() {
    assert_html_eq!({
        a #"foo" {
            "bar"
        }
    } => "<a id=\"foo\">bar</a>");

    assert_html_eq!({
        a #foo {
            "bar"
        }
    } => "<a id=\"foo\">bar</a>");

    assert_html_eq!({
        a #(3) {
            "bar"
        }
    } => "<a id=\"3\">bar</a>");
}

#[test]
fn id_toggle() {
    assert_html_eq!({
        a #[Some(3)] {
            "bar"
        }
    } => "<a id=\"3\">bar</a>");

    assert_html_eq!({
        a #[Option::<i32>::None] {
            "bar"
        }
    } => "<a>bar</a>");
}

#[test]
fn class_literal() {
    assert_html_eq!({
        ."headers" {}
    } => "<div class=\"headers\"></div>");
}

#[test]
fn class_ident() {
    assert_html_eq!({
        .headers {}
    } => "<div class=\"headers\"></div>");

    assert_html_eq!({
        .head-ers {}
    } => "<div class=\"head-ers\"></div>");
}

#[test]
fn id_literal() {
    assert_html_eq!({
        #"headers" {}
    } => "<div id=\"headers\"></div>");
}

#[test]
fn id_ident() {
    assert_html_eq!({
        #headers {}
    } => "<div id=\"headers\"></div>");

    assert_html_eq!({
        #head-ers {}
    } => "<div id=\"head-ers\"></div>");
}
