pub trait Discriminant: Sized {
    fn as_discriminant(&self) -> i32;

    fn from_discriminant(discriminant: i32) -> Option<Self>;
}

impl Discriminant for i32 {
    fn as_discriminant(&self) -> i32 {
        *self
    }

    fn from_discriminant(discriminant: i32) -> Option<Self> {
        Some(discriminant)
    }
}

impl Discriminant for u32 {
    fn as_discriminant(&self) -> i32 {
        *self as _
    }

    fn from_discriminant(discriminant: i32) -> Option<Self> {
        Some(discriminant as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_discriminant() {
        assert_eq!((-1i32).as_discriminant(), -1i32);
        assert_eq!(2i32.as_discriminant(), 2i32);
        assert_eq!(3i32.as_discriminant(), 3i32);
        assert_eq!(251252114i32.as_discriminant(), 251252114i32);
        assert_eq!(i32::MAX.as_discriminant(), i32::MAX);
        assert_eq!(i32::MIN.as_discriminant(), i32::MIN);

        assert_eq!(1u32.as_discriminant(), 1i32);
        assert_eq!(2u32.as_discriminant(), 2i32);
        assert_eq!(3u32.as_discriminant(), 3i32);
        assert_eq!(251252114u32.as_discriminant(), 251252114i32);
        assert_eq!(u32::MAX.as_discriminant(), -1i32);
        assert_eq!(u32::MIN.as_discriminant(), 0i32);
    }

    #[test]
    fn from_discriminant() {
        assert_eq!(i32::from_discriminant(-1i32), Some(-1i32));
        assert_eq!(i32::from_discriminant(2i32), Some(2i32));
        assert_eq!(i32::from_discriminant(3i32), Some(3i32));
        assert_eq!(i32::from_discriminant(251252114i32), Some(251252114i32));
        assert_eq!(i32::from_discriminant(i32::MAX), Some(i32::MAX));
        assert_eq!(i32::from_discriminant(i32::MIN), Some(i32::MIN));

        assert_eq!(u32::from_discriminant(-1i32), Some(4294967295));
        assert_eq!(u32::from_discriminant(2i32), Some(2u32));
        assert_eq!(u32::from_discriminant(3i32), Some(3u32));
        assert_eq!(u32::from_discriminant(251252114i32), Some(251252114u32));
        assert_eq!(u32::from_discriminant(i32::MAX), Some(2147483647u32));
        assert_eq!(u32::from_discriminant(i32::MIN), Some(2147483648u32));
    }
}
