use crate::assert_html_eq;

use lfml::html;

#[test]
fn markup_body() {
    let x = "foobar";

    assert_html_eq!({
        (x)
    } => "foobar");

}
