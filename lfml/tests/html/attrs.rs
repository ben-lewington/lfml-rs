use crate::assert_html_eq;

use lfml::html;

#[test]
fn without_value() {
    assert_html_eq!({
        a foo {}
    } => "<a foo></a>");

    assert_html_eq!({
        a foo bar {}
    } => "<a foo bar></a>");
}
