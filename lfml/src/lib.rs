pub use lfml_escape::{escape_string, escape_to_string};
pub use lfml_macros::{html, EmbedAsAttrs};

pub trait EmbedAsAttrs {
    fn raw(&self) -> String;
}

pub struct Escaped<T>(pub T);

impl<T: std::fmt::Display> Escaped<T> {
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }
}

pub trait Escapable {
    fn markup(&self) -> Escaped<String> {
        let mut buf = String::new();
        self.markup_to_string(&mut buf);
        Escaped(buf)
    }

    fn markup_to_string(&self, buf: &mut String) {
        buf.push_str(&self.markup().into_string())
    }
}

impl<T: std::fmt::Display> Escapable for Escaped<T> {
    fn markup_to_string(&self, buf: &mut String) {
        buf.push_str(&self.into_string())
    }
}

impl Escapable for str {
    fn markup_to_string(&self, buf: &mut String) {
        escape_to_string(self, buf);
    }
}

impl Escapable for String {
    fn markup_to_string(&self, buf: &mut String) {
        str::markup_to_string(self, buf)
    }
}

impl<'a, T: Escapable + ?Sized> Escapable for &'a T {
    fn markup_to_string(&self, buf: &mut String) {
        T::markup_to_string(self, buf);
    }
}

impl<'a, T: Escapable + ?Sized> Escapable for &'a mut T {
    fn markup_to_string(&self, buf: &mut String) {
        T::markup_to_string(self, buf);
    }
}

macro_rules! impl_render_for_integer_types {
    ($($ty:ty)*) => {
        $(
            impl Escapable for $ty {
                fn markup_to_string(&self, w: &mut String) {
                    w.push_str(itoa::Buffer::new().format(*self));
                }
            }
        )*
    };
}

impl_render_for_integer_types! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}
