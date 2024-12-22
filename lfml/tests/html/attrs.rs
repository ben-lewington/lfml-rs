use crate::assert_html_eq;

#[test]
fn without_value() {
    assert_html_eq!({
        a foo {}
    } => "<a foo></a>");

    assert_html_eq!({
        a foo bar {}
    } => "<a foo bar></a>");
}

#[test]
fn with_value() {
    assert_html_eq!({
        a foo="bar" {}
    } => "<a foo=\"bar\"></a>");

    assert_html_eq!({
        a foo=3 bar=3.5 baz=true bat='a' car=b'a' cat=b"Hello \xF0\x90\x80World" {}
    } => "<a foo=\"3\" bar=\"3.5\" baz=\"true\" bat=\"a\" car=\"a\" cat=\"Hello �World\"></a>");
}

#[test]
fn mix_of_two_types_of_attrs() {
    assert_html_eq!({
        a foo="bar" baz {}
    } => "<a foo=\"bar\" baz></a>");

    assert_html_eq!({
        a foo bar="baz" {}
    } => "<a foo bar=\"baz\"></a>");
}

#[test]
fn self_closing_tags() {
    assert_html_eq!({
        a foo;
    } => "<a foo>");

    assert_html_eq!({
        a foo; b {}
    } => "<a foo><b></b>");

    assert_html_eq!({
        a foo="bar"; b {}
    } => "<a foo=\"bar\"><b></b>");
}

#[test]
fn attr_names_with_hyphens() {
    assert_html_eq!({
        a foo-bar;
    } => "<a foo-bar>");

    assert_html_eq!({
        a-b foo-bar="baz";
    } => "<a-b foo-bar=\"baz\">");
}

#[test]
fn attr_id_and_class() {
    assert_html_eq!({
        ."" {}
    } => "<div class=\"\"></div>");

    assert_html_eq!({
        a-b foo-bar="baz";
    } => "<a-b foo-bar=\"baz\">");
}
