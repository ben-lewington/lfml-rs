use crate::assert_html_eq;

#[test]
fn string() {
    assert_html_eq!({ "du\tcks" } => "du\tcks");

    assert_html_eq!({ 'a' } => "a");
}

#[test]
fn concatenation() {
    assert_html_eq!({ "du\tcks" "-23" "3.14\n" "geese" } => "du\tcks-233.14\ngeese");
}

#[test]
fn not_string_that_impl_display() {
    assert_html_eq!({ 3 } => "3");

    assert_html_eq!({ true " " false } => "true false");
}

#[test]
fn negative_numbers() {
    assert_html_eq!({ -1 } => "-1");

    assert_html_eq!({ -1.5 -3.75 } => "-1.5-3.75");
}

#[test]
fn byte_strings() {
    assert_html_eq!({ b"foobar" } => "foobar");

    assert_html_eq!({ b"Hello \xF0\x90\x80World" } => "Hello �World");
}

#[test]
fn byte() {
    assert_html_eq!({ b'f' } => "f");
}

#[test]
fn float() {
    assert_html_eq!({ 1f64 2f32 } => "12");

    assert_html_eq!({ 1.0 2.5 3.25f64 } => "12.53.25");
}

#[test]
fn is_escaped() {
    assert_html_eq!({ "<>" } => "&lt;&gt;")
}
