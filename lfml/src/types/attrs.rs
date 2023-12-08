pub trait MarkupAttrs {
    fn raw(&self) -> String;
}

impl<'a, T: MarkupAttrs> MarkupAttrs for &'a T {
    fn raw(&self) -> String {
        MarkupAttrs::raw(*self)
    }
}

impl<'a, T: MarkupAttrs> MarkupAttrs for &'a mut T {
    fn raw(&self) -> String {
        MarkupAttrs::raw(*self)
    }
}
