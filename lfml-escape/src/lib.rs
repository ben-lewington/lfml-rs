extern crate alloc;
use alloc::string::String;

pub fn escape_to_string(input: &str, output: &mut String) {
    for b in input.bytes() {
        match b {
            b'&' => output.push_str("&amp;"),
            b'<' => output.push_str("&lt;"),
            b'>' => output.push_str("&gt;"),
            b'"' => output.push_str("&quot;"),
            // Safety: `input` is valid utf-8.
            _ => unsafe { output.as_mut_vec().push(b) },
        }
    }
}

pub fn escape_string(input: &str) -> String {
    let mut s = String::new();
    escape_to_string(input, &mut s);
    s
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn escape_works_as_expected() {
        let mut s = String::new();
        escape_to_string("<script>BadThings()</script>", &mut s);
        assert_eq!(s, "&lt;script&gt;BadThings()&lt;/script&gt;");
    }
}
