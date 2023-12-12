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

#[test]
fn restrict_attribute() {
    #[derive(Spread)]
    #[prefix = "hx"]
    #[tags(only(a, b, c))]
    enum HxControl<'a> {
        Get {
            get: &'a str,
            target: &'a str,
            swap: &'a str,
        },
        Post {
            post: &'a str,
            target: &'a str,
            swap: &'a str,
        },
    }

    assert_html_eq!({
        a@(HxControl::Get { get: "/a", target: ".main", swap: "outerHTML" }) { "A" }
        a@(HxControl::Post { post: "/a", target: ".main", swap: "outerHTML" }) { "A" }
    } => "<a hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</a><a hx-post=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</a>");

    assert_html_eq!({
        b@(HxControl::Get { get: "/a", target: ".main", swap: "outerHTML" }) { "A" }
        b@(HxControl::Post { post: "/a", target: ".main", swap: "outerHTML" }) { "A" }
    } => "<b hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</b><b hx-post=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</b>");
    assert_html_eq!({
        c@(HxControl::Get { get: "/a", target: ".main", swap: "outerHTML" }) { "A" }
        c@(HxControl::Post { post: "/a", target: ".main", swap: "outerHTML" }) { "A" }
    } => "<c hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</c><c hx-post=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</c>");
}

#[test]
fn multiple() {
    #[derive(Spread)]
    #[tags(only(a))]
    struct Foo<'a> {
        target: &'a str,
    }

    #[derive(Spread)]
    #[tags(only(a))]
    enum Bar<'a> {
        Baz { get: &'a str },
    }

    let x = Bar::Baz { get: "/" };

    assert_html_eq!({
        a @(Foo { target: ".main" }) @(x) {
            "A"
        }
    } => "<a target=\".main\" get=\"/\">A</a>");
}

#[test]
fn prefix_precedent() {
    #[derive(Spread)]
    #[prefix]
    enum Bar<'a> {
        Baz {
            get: &'a str,
        },
        #[prefix = "x-data"]
        Bat {
            get: &'a str,
        },
    }

    let x = Bar::Baz { get: "/" };

    assert_html_eq!({
        a @(x) {
            "A"
        }
        a @(Bar::Bat { get: "/" });
    } => "<a data-get=\"/\">A</a><a x-data-get=\"/\">");
}

#[test]
fn option_type_with_toggle_syntax() {
    #[derive(Spread)]
    #[tags(include(a))]
    struct Foo<'a> {
        target: &'a str,
    }

    let x = Some(Foo { target: ".main" });

    assert_html_eq!({
        a @[x] {
            "A"
        }
    } => "<a target=\".main\">A</a>");

    let x: Option<Foo<'_>> = None;

    assert_html_eq!({
        a @[x] {
            "A"
        }
    } => "<a>A</a>");
}
