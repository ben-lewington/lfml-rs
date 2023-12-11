#[test]
fn basic() {
    #[derive(lfml::Spread)]
    #[tags(include(a, b, c), exclude(a), only(d))]
    enum Foo<'a> {
        Bar { a: &'a str },
    }
}
