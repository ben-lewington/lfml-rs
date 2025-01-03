use lfml::{NameOnly, Spread};

#[test]
fn basic() {
    #[derive(Spread)]
    struct A<'a> {
        foo: i32,
        bar: &'a str,
    }

    let y = A { foo: 0, bar: "a" };

    assert_eq!(Spread::raw(&y), " foo=\"0\" bar=\"a\"");
}

#[test]
fn escape_values() {
    #[derive(Spread)]
    struct A {
        #[escape_value]
        bar: String,
    }

    let y = A {
        bar: "<a></a>".into(),
    };

    assert_eq!(Spread::raw(&y), " bar=\"&lt;a&gt;&lt;/a&gt;\"");
}

#[test]
fn global_prefix() {
    #[derive(Spread)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " data-foo=\"a\" data-bar=\"a\"");

    #[derive(Spread)]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " x-data-foo=\"a\" x-data-bar=\"a\"");
}

#[test]
fn global_suffix() {
    #[derive(Spread)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " foo-attr=\"a\" bar-attr=\"a\"");
}

#[test]
fn rename_field() {
    #[derive(Spread)]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " foo=\"a\" baz=\"a\"");
}

#[test]
fn rename_overrides_prefix_and_suffix() {
    #[derive(Spread)]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " data-foo=\"a\" baz=\"a\"");

    #[derive(Spread)]
    #[suffix = "attr"]
    struct B<'a> {
        foo: &'a str,
        #[rename = "baz"]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " foo-attr=\"a\" baz=\"a\"");
}

#[test]
fn global_prefix_and_suffix() {
    #[derive(Spread)]
    #[suffix = "attr"]
    #[prefix]
    struct A<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "a" };

    assert_eq!(Spread::raw(&y), " data-foo-attr=\"a\" data-bar-attr=\"a\"");

    #[derive(Spread)]
    #[suffix = "attr"]
    #[prefix = "x-data"]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(
        Spread::raw(&y),
        " x-data-foo-attr=\"a\" x-data-bar-attr=\"a\""
    );
}

#[test]
fn blanda_uppa() {
    #[derive(Spread)]
    #[suffix = "attr"]
    struct A<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = A { foo: "a", bar: "<" };

    assert_eq!(Spread::raw(&y), " foo-attr=\"a\" bar-attr=\"&lt;\"");

    #[derive(Spread)]
    #[prefix]
    struct B<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "<" };

    assert_eq!(Spread::raw(&y), " data-foo=\"a\" data-bar=\"&lt;\"");

    #[derive(Spread)]
    #[prefix]
    #[suffix = "attr"]
    struct C<'a> {
        foo: &'a str,
        #[escape_value]
        bar: &'a str,
    }

    let y = C { foo: "a", bar: "<" };

    assert_eq!(
        Spread::raw(&y),
        " data-foo-attr=\"a\" data-bar-attr=\"&lt;\""
    );
}

#[test]
fn display_trait_bound_is_delegated_for_generic_types() {
    #[derive(Spread)]
    struct A<T> {
        foo: T,
    }
    let _: A<()> = A { foo: () };
}

#[test]
fn option_fields_are_handled() {
    #[derive(Spread)]
    struct A {
        foo: std::option::Option<String>,
        bar: Option<i32>,
    }

    let y = A {
        foo: Some("a".into()),
        bar: None,
    };

    assert_eq!(Spread::raw(&y), " foo=\"a\"");

    let y = A {
        foo: None,
        bar: None,
    };

    assert_eq!(Spread::raw(&y), "");

    let y = A {
        foo: Some("a".into()),
        bar: Some(0),
    };

    assert_eq!(Spread::raw(&y), " foo=\"a\" bar=\"0\"");
}

#[test]
fn option_fields_work_with_generics() {
    #[derive(Spread)]
    struct B<T> {
        foo: Option<T>,
    }

    let y = B { foo: Some("a") };

    assert_eq!(Spread::raw(&y), " foo=\"a\"");
}

#[test]
fn option_fields_can_be_escaped() {
    #[derive(Spread)]
    struct A<'a> {
        #[escape_value]
        foo: Option<&'a str>,
    }

    let y = A { foo: Some("<") };

    assert_eq!(Spread::raw(&y), " foo=\"&lt;\"");
}

#[test]
fn spread_with_unit_nameonly_field_produces_valueless_tag() {
    #[derive(Spread)]
    #[tags(only(a))]
    struct A {
        #[escape_value]
        _foo: lfml::NameOnly,
    }

    let y = A { _foo: NameOnly };

    assert_eq!(Spread::raw(&y), " _foo");
}

#[test]
fn spread_with_option_unit_nameonly_field_produces_valueless_tag() {
    #[derive(Spread)]
    #[tags(only(a))]
    struct A {
        #[escape_value]
        foo: Option<lfml::NameOnly>,
    }

    let y = A {
        foo: Some(NameOnly),
    };

    assert_eq!(Spread::raw(&y), " foo");

    #[derive(Spread)]
    #[tags(only(a))]
    struct B {
        #[escape_value]
        foo: Option<lfml::NameOnly>,
    }

    let y = B { foo: None };

    assert_eq!(Spread::raw(&y), "");
}
