use crate::assert_html_eq;
use lfml_macros::Spread;

#[test]
fn basic() {
    #[derive(Spread)]
    #[prefix = "hx"]
    enum HxControl<'a> {
        Get { get: &'a str },
        Post { post: &'a str },
    }

    let x = HxControl::Get { get: "/" };

    assert_html_eq!({
        a@(x) { "A" }
    } => "<a hx-get=\"/\">A</a>");

    let x = HxControl::Post { post: "/" };

    assert_html_eq!({
        a@(x) { "A" }
    } => "<a hx-post=\"/\">A</a>");
}
