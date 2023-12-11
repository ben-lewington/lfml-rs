use lfml_macros::Spread;

#[test]
fn basic() {
    #[derive(Spread)]
    #[prefix = "hx"]
    enum HxControl<'a> {
        Get { get: &'a str },
        Post { post: &'a str },
    }
}
