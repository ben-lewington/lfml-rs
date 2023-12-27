mod html;
mod spread;

#[macro_export]
macro_rules! assert_html_eq {
    ($markup:tt => $output:literal) => {
        assert_eq!({ lfml::html! $markup }.as_string(), $output)
    };
}
