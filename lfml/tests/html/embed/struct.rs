use crate::assert_html_eq;

use lfml::MarkupAttrs;

#[test]
fn basic() {
    #[derive(MarkupAttrs)]
    #[prefix = "hx"]
    struct HxGet<'a> {
        get: &'a str,
        target: &'a str,
        swap: &'a str,
    }

    let x = HxGet {
        get: "/a",
        target: ".main",
        swap: "outerHTML",
    };

    assert_html_eq!({
        a@x { "A" }
    } => "<a hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</a>");

    assert_html_eq!({
        div@{
            HxGet {
                get: "/a",
                target: ".main",
                swap: "outerHTML",
            }
        } {
            "A"
        }
    } => "<div hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</div>");
}

#[test]
fn restrict_attribute() {
    #[derive(MarkupAttrs)]
    #[prefix = "hx"]
    #[tags(a, b, c)]
    struct HxGet<'a> {
        get: &'a str,
        target: &'a str,
        swap: &'a str,
    }

    let x = HxGet {
        get: "/a",
        target: ".main",
        swap: "outerHTML",
    };

    assert_html_eq!({
        a@x { "A" }
    } => "<a hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</a>");

    assert_html_eq!({
        b@{
            HxGet {
                get: "/a",
                target: ".main",
                swap: "outerHTML",
            }
        } {
            "A"
        }
    } => "<b hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</b>");

    assert_html_eq!({
        c@x { "A" }
    } => "<c hx-get=\"/a\" hx-target=\".main\" hx-swap=\"outerHTML\">A</c>");
}

#[test]
fn embed_multiple_structs_on_valid_tag() {
    #[derive(MarkupAttrs)]
    #[tags(a)]
    struct Foo<'a> {
        target: &'a str,
    }

    #[derive(MarkupAttrs)]
    #[tags(a)]
    struct Bar<'a> {
        get: &'a str,
    }

    let x = Bar { get: "/" };

    assert_html_eq!({
        a @( Foo { target: ".main" } ) @x {
            "A"
        }
    } => "<a target=\".main\" get=\"/\">A</a>");
}

#[test]
fn embed_option_type_with_toggle_syntax() {
    #[derive(MarkupAttrs)]
    #[tags(a)]
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
