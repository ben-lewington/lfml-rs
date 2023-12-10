use lfml::MarkupAttrs;

#[test]
fn basic() {
    #[derive(MarkupAttrs)]
    struct A<'a> {
        foo: i32,
        bar: &'a str,
    }

    let y = A { foo: 0, bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"0\" bar=\"a\"");
}

#[test]
fn escape_values() {
    #[derive(MarkupAttrs)]
    struct A {
        #[escape_value]
        bar: String,
    }

    let y = A {
        bar: "<a></a>".into(),
    };

    assert_eq!(MarkupAttrs::raw(&y), " bar=\"&lt;a&gt;&lt;/a&gt;\"");
}

#[test]
fn global_prefix() {
    #[derive(MarkupAttrs)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " data-foo=\"a\" data-bar=\"a\"");

    #[derive(MarkupAttrs)]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " x-data-foo=\"a\" x-data-bar=\"a\"");
}

#[test]
fn global_suffix() {
    #[derive(MarkupAttrs)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " foo-attr=\"a\" bar-attr=\"a\"");
}

#[test]
fn rename_field() {
    #[derive(MarkupAttrs)]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"a\" baz=\"a\"");
}

#[test]
fn rename_overrides_prefix_and_suffix() {
    #[derive(MarkupAttrs)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " data-foo=\"a\" baz=\"a\"");

    #[derive(MarkupAttrs)]
    #[suffix = "attr"]
    struct B<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(MarkupAttrs::raw(&y), " foo-attr=\"a\" baz=\"a\"");
}

#[test]
fn global_prefix_and_suffix() {
    #[derive(MarkupAttrs)]
    #[suffix = "attr"]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(
        MarkupAttrs::raw(&y),
        " data-foo-attr=\"a\" data-bar-attr=\"a\""
    );

    #[derive(MarkupAttrs)]
    #[suffix = "attr"]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(
        MarkupAttrs::raw(&y),
        " x-data-foo-attr=\"a\" x-data-bar-attr=\"a\""
    );
}

#[test]
fn blanda_uppa() {
    #[derive(MarkupAttrs)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "<" };

    assert_eq!(MarkupAttrs::raw(&y), " foo-attr=\"a\" bar-attr=\"&lt;\"");

    #[derive(MarkupAttrs)]
    #[prefix]
    struct B<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "<" };

    assert_eq!(MarkupAttrs::raw(&y), " data-foo=\"a\" data-bar=\"&lt;\"");

    #[derive(MarkupAttrs)]
    #[prefix]
    #[suffix = "attr"]
    struct C<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = C { foo: "a", bar: "<" };

    assert_eq!(
        MarkupAttrs::raw(&y),
        " data-foo-attr=\"a\" data-bar-attr=\"&lt;\""
    );
}

#[test]
fn display_trait_bound_is_delegated_for_generic_types() {
    #[derive(MarkupAttrs)]
    struct A<T> {
        foo: T,
    }
}

#[test]
fn option_fields_are_handled() {
    #[derive(MarkupAttrs)]
    struct A {
        foo: std::option::Option<String>,
        bar: Option<i32>,
    }

    let y = A {
        foo: Some("a".into()),
        bar: None,
    };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"a\"");

    let y = A {
        foo: None,
        bar: None,
    };

    assert_eq!(MarkupAttrs::raw(&y), "");

    let y = A {
        foo: Some("a".into()),
        bar: Some(0),
    };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"a\" bar=\"0\"");
}

#[test]
fn option_fields_work_with_generics() {
    #[derive(MarkupAttrs)]
    struct B<T> {
        foo: Option<T>,
    }

    let y = B { foo: Some("a") };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"a\"");
}

#[test]
fn option_fields_can_be_escaped() {
    #[derive(MarkupAttrs)]
    struct A<'a> {
        #[escape_value]
        foo: Option<&'a str>,
    }

    let y = A { foo: Some("<") };

    assert_eq!(MarkupAttrs::raw(&y), " foo=\"&lt;\"");
}
