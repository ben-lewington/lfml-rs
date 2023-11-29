use lfml::EmbedAsAttrs;

#[test]
fn basic_attrs() {
    #[derive(EmbedAsAttrs)]
    struct Datum<'a> {
        foo: i32,
        bar: &'a str,
    }

    let y = Datum { foo: 0, bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " foo=\"0\" bar=\"a\" ");
}

#[test]
fn global_prefixes() {
    #[derive(EmbedAsAttrs)]
    struct A<T: core::fmt::Display> {
        #[escape_value]
        bar: T,
    }

    let y = A { bar: "<a></a>" };

    assert_eq!(EmbedAsAttrs::raw(&y), " bar=\"&lt;a&gt;&lt;/a&gt;\" ");

    #[derive(EmbedAsAttrs)]
    #[prefix]
    struct B<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = B { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " data-foo=\"a\" data-bar=\"a\" ");

    #[derive(EmbedAsAttrs)]
    #[prefix = "x-data"]
    struct C<'a> {
        foo: &'a str,
        bar: &'a str,
    }

    let y = C { foo: "a", bar: "a" };

    assert_eq!(EmbedAsAttrs::raw(&y), " x-data-foo=\"a\" x-data-bar=\"a\" ");
}
