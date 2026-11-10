pub enum Reference<'b, 'c, T: ?Sized> {
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T: ?Sized> AsRef<T> for Reference<'b, 'c, T> {
    fn as_ref(&self) -> &T {
        match self {
            Reference::Borrowed(v) => v,
            Reference::Copied(v) => v,
        }
    }
}
