use crate::assert_html_eq;

use lfml::html;

#[test]
fn simple() {
    assert_html_eq!({a {}} => "<a></a>")
}
