use crate::assert_html_eq;

#[test]
fn slot_simple() {
    let x = "foobar";

    assert_html_eq!({
        (x)
    } => "foobar");

    let x = 3;

    assert_html_eq!({
        (x)
    } => "3");

    let x = lfml::html! {
        a {
            "Foo"
        }
    };

    assert_html_eq!({
        (x)
    } => "<a>Foo</a>");
}

#[test]
fn slot_match() {
    let x = Some(3);
    assert_html_eq!({
        @match x {
            Some(i) => {
                a { (i) }
            }
            None => {
                a { 0 }
            }
        }
    } => "<a>3</a>");

    let x: Option<usize> = None;
    assert_html_eq!({
        @match x {
            Some(i) => {
                a { (i) }
            }
            None => {
                a { 0 }
            }
        }
    } => "<a>0</a>");
}
