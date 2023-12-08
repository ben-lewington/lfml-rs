mod derive;
mod html;

#[macro_export]
macro_rules! assert_html_eq {
    ($markup:tt => $output:literal) => {
        assert_eq!({html! $markup}.as_string(), $output)
    };
}
