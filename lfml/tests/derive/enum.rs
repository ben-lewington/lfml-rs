#[test]
fn basic() {
    #[derive(lfml::Spread)]
    #[tags(include(a, b, c))]
    enum A<'a> {
        Bar { a: &'a str },
        Baz { b: &'a str },
    }

    let y = A::Bar { a: "a" };

    assert_eq!(lfml::Spread::raw(&y), " a=\"a\"");

    let y = A::Baz { b: "a" };

    assert_eq!(lfml::Spread::raw(&y), " b=\"a\"");
}
