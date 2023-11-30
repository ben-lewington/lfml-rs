use lfml::EmbedAsAttrs;

#[test]
fn basic() {
    #[derive(EmbedAsAttrs)]
    struct A<'a> {
        foo: i32,
        bar: &'a str,
    }

    let y = A { foo: 0, bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo=\"0\" bar=\"a\" ");
}

#[test]
fn escape_values() {
    #[derive(EmbedAsAttrs)]
    struct A {
        #[escape_value]
        bar: String,
    }

    let y = A {
        bar: "<a></a>".into(),
    };

    assert_eq!(EmbedAsAttrs::raw(&y), " bar=\"&lt;a&gt;&lt;/a&gt;\" ");
}

#[test]
fn global_prefix() {
    #[derive(EmbedAsAttrs)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " data-foo=\"a\" data-bar=\"a\" ");

    #[derive(EmbedAsAttrs)]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " x-data-foo=\"a\" x-data-bar=\"a\" ");
}

#[test]
fn global_suffix() {
    #[derive(EmbedAsAttrs)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo-attr=\"a\" bar-attr=\"a\" ");
}

#[test]
fn rename_field() {
    #[derive(EmbedAsAttrs)]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo=\"a\" baz=\"a\" ");
}

#[test]
fn rename_overrides_prefix_and_suffix() {
    #[derive(EmbedAsAttrs)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " data-foo=\"a\" baz=\"a\" ");

    #[derive(EmbedAsAttrs)]
    #[suffix = "attr"]
    struct B<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo-attr=\"a\" baz=\"a\" ");
}

#[test]
fn global_prefix_and_suffix() {
    #[derive(EmbedAsAttrs)]
    #[suffix = "attr"]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(
        EmbedAsAttrs::raw(&y),
        " data-foo-attr=\"a\" data-bar-attr=\"a\" "
    );

    #[derive(EmbedAsAttrs)]
    #[suffix = "attr"]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(
        EmbedAsAttrs::raw(&y),
        " x-data-foo-attr=\"a\" x-data-bar-attr=\"a\" "
    );
}

#[test]
fn blanda_uppa() {
    #[derive(EmbedAsAttrs)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "<" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo-attr=\"a\" bar-attr=\"&lt;\" ");

    #[derive(EmbedAsAttrs)]
    #[prefix]
    struct B<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "<" };

    assert_eq!(EmbedAsAttrs::raw(&y), " data-foo=\"a\" data-bar=\"&lt;\" ");

    #[derive(EmbedAsAttrs)]
    #[prefix]
    #[suffix = "attr"]
    struct C<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = C { foo: "a", bar: "<" };

    assert_eq!(
        EmbedAsAttrs::raw(&y),
        " data-foo-attr=\"a\" data-bar-attr=\"&lt;\" "
    );
}

#[test]
fn display_trait_bound_is_delegated_for_generic_types() {
    #[derive(EmbedAsAttrs)]
    struct A<T> {
        foo: T,
    }
}

#[test]
fn option_fields_are_handled() {
    #[derive(EmbedAsAttrs)]
    struct A {
        foo: std::option::Option<String>,
        bar: Option<i32>,
    }

    let y = A {
        foo: Some("a".into()),
        bar: None,
    };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo=\"a\" ");

    let y = A {
        foo: None,
        bar: None,
    };

    assert_eq!(EmbedAsAttrs::raw(&y), " ");

    let y = A {
        foo: Some("a".into()),
        bar: Some(0),
    };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo=\"a\" bar=\"0\" ");
}
