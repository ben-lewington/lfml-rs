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

#[test]
fn slot_trailing_comma() {
    let x = Some(3);

    assert_html_eq!({
        @match x {
            Some(i) => {
                a { (i) }
            },
            None => {
                a { 0 }
            },
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

#[test]
fn slot_for() {
    assert_html_eq!({
        @for i in 0..=2 {
            a { (i) }
        }
    } => "<a>0</a><a>1</a><a>2</a>");
}

#[test]
fn slot_for_brace_group_before_in_kw() {
    struct A {
        a: i32,
    }

    let x: Vec<_> = (0..).take(5).map(|a| A { a }).collect();

    assert_html_eq!({
        @for A { a: i } in x {
            a { (i) }
        }
    } => "<a>0</a><a>1</a><a>2</a><a>3</a><a>4</a>");
}

#[test]
fn slot_if() {
    let a = 1;

    assert_html_eq!({
        @if a == 1 {
            a { (a) }
        } @else if a == 2 {
            a { "foo " (a) }
        } @else {
            a { "snake eyes!" }
        }
    } => "<a>1</a>");

    let a = 2;

    assert_html_eq!({
        @if a == 1 {
            a { (a) }
        } @else if a == 2 {
            a { "foo " (a) }
        } @else {
            a { "snake eyes!" }
        }
    } => "<a>foo 2</a>");

    let a = 3;

    assert_html_eq!({
        @if a == 1 {
            a { (a) }
        } @else if a == 2 {
            a { "foo " (a) }
        } @else {
            a { "snake eyes!" }
        }
    } => "<a>snake eyes!</a>");
}
