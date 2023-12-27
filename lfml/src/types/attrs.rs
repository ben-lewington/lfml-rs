pub trait Spread {
    fn raw(&self) -> String;
}

pub struct NameOnly;

impl<'a, T: Spread> Spread for &'a T {
    fn raw(&self) -> String {
        Spread::raw(*self)
    }
}

impl<'a, T: Spread> Spread for &'a mut T {
    fn raw(&self) -> String {
        Spread::raw(*self)
    }
}
