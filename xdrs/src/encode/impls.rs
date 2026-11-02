use std::{borrow::Cow, io::Write, marker::PhantomData};

use bytes::Bytes;

use crate::{ExpectedSize, encode::Encode, error::Error, padding};

macro_rules! impl_num {
    ($($ty:ty),*$(,)?) => {
        $(
        impl Encode for $ty {
            #[inline]
            fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_all(&self.to_be_bytes())
                    .map_err(Error::io)
                    .map(|_| std::mem::size_of::<$ty>())
            }
        }
        )*
    };
}

impl_num!(i32, u32, i64, u64, f32, f64);

impl Encode for bool {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (*self as i32).encode(writer)
    }
}

impl Encode for str {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        if !self.is_ascii() {
            return Err(Error::invalid_value(
                "not ascii string",
                "valid ascii string",
            ));
        }

        self.as_bytes().encode(writer)
    }
}

impl Encode for &'_ str {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (*self).encode(writer)
    }
}

impl Encode for String {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        self.as_str().encode(writer)
    }
}

impl<T: Encode> Encode for [T] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut wrote = 0;

        wrote += write_size(writer, self.len())?;
        wrote += write_seq(writer, self)?;
        wrote += write_padding(writer, wrote)?;

        Ok(wrote)
    }
}

impl<T: Encode> Encode for &'_ [T] {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (*self).encode(writer)
    }
}

impl Encode for [u8] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut wrote = 0;

        wrote += write_size(writer, self.len())?;
        wrote += write_bytes(writer, self)?;
        wrote += write_padding(writer, wrote)?;

        Ok(wrote)
    }
}

impl Encode for &'_ [u8] {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (*self).encode(writer)
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut wrote = 0;

        wrote += write_seq(writer, self)?;
        wrote += write_padding(writer, wrote)?;

        Ok(wrote)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut wrote = 0;

        wrote += write_bytes(writer, self)?;
        wrote += write_padding(writer, wrote)?;

        Ok(wrote)
    }
}

impl<T: Encode> Encode for Vec<T> {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        self.as_slice().encode(writer)
    }
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        self.as_slice().encode(writer)
    }
}

impl Encode for Bytes {
    #[inline]
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        self.as_ref().encode(writer)
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        match self {
            None => false.encode(writer),
            Some(v) => {
                let mut wrote = 0;

                wrote += true.encode(writer)?;
                wrote += v.encode(writer)?;

                Ok(wrote)
            }
        }
    }
}

impl<T: Encode> Encode for Box<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (**self).encode(writer)
    }
}

impl<'a, T> Encode for Cow<'a, T>
where
    T: ?Sized + ToOwned,
    T: Encode,
    T::Owned: Encode,
{
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        (**self).encode(writer)
    }
}

impl Encode for () {
    #[inline]
    fn encode<W: Write>(&self, _writer: &mut W) -> Result<usize, Error> {
        Ok(0)
    }
}

impl<T> Encode for PhantomData<T> {
    #[inline]
    fn encode<W: Write>(&self, _writer: &mut W) -> Result<usize, Error> {
        Ok(0)
    }
}

#[inline]
pub(crate) fn write_size<W>(writer: &mut W, sz: usize) -> Result<usize, Error>
where
    W: Write,
{
    check_size_var(sz, u32::MAX as usize)?;

    // SAFETY: sz already checked above
    let sz = sz as u32;

    sz.encode(writer)
}

#[inline]
pub(crate) fn write_padding<W>(
    writer: &mut W,
    sz: usize,
) -> Result<usize, Error>
where
    W: Write,
{
    let mut wrote = 0;

    let p = padding(sz);
    if !p.is_empty() {
        writer.write_all(p).map_err(Error::io)?;
        wrote += p.len();
    }

    Ok(wrote)
}

#[inline]
pub(crate) fn write_seq<'a, W, I, T>(
    writer: &mut W,
    iter: I,
) -> Result<usize, Error>
where
    W: Write,
    I: IntoIterator<Item = &'a T>,
    T: Encode + 'a,
{
    let mut wrote = 0;

    for it in iter.into_iter() {
        wrote += it.encode(writer)?;
    }

    Ok(wrote)
}

#[inline]
pub(crate) fn write_bytes<W>(
    writer: &mut W,
    bytes: &[u8],
) -> Result<usize, Error>
where
    W: Write,
{
    writer.write_all(bytes).map_err(Error::io)?;
    Ok(bytes.len())
}

#[inline]
pub(crate) fn check_size_var(sz: usize, maxsz: usize) -> Result<(), Error> {
    if sz <= maxsz {
        Ok(())
    } else {
        Err(Error::invalid_size(sz, ExpectedSize::Max(maxsz)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_encode<T: Encode>(val: T, exp: (&[u8], usize)) {
        let mut writer = Vec::new();
        let n = T::encode(&val, &mut writer).unwrap();
        assert_eq!((writer.as_slice(), n), exp);
    }

    #[rustfmt::skip]
    #[test]
    fn numbers() {
        assert_encode::<i32>(42, (&[0x00, 0x00, 0x00, 0x2A], 4));
        assert_encode::<i32>(-42, (&[0xFF, 0xFF, 0xFF, 0xD6], 4));
        assert_encode::<u32>(42, (&[0x00, 0x00, 0x00, 0x2A], 4));

        assert_encode::<i64>(42, (&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], 8));
        assert_encode::<i64>(-42, (&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD6], 8));
        assert_encode::<u64>(42, (&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], 8));

        assert_encode::<f32>(42.42, (&[0x42, 0x29, 0xAE, 0x14], 4));
        assert_encode::<f64>(42.42, (&[0x40, 0x45, 0x35, 0xC2, 0x8F, 0x5C, 0x28, 0xF6], 8));

        assert_encode::<bool>(false, (&[0x00, 0x00, 0x00, 0x00], 4));
        assert_encode::<bool>(true, (&[0x00, 0x00, 0x00, 0x01], 4));
    }

    #[rustfmt::skip]
    #[test]
    fn opaque() {
        assert_encode::<&[u8]>(&[0x00, 0x01, 0x02, 0x03, 0xFF],
        (&[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ], 12)
        );
        assert_encode::<Vec<u8>>(vec![0x00, 0x01, 0x02, 0x03, 0xFF],
        (&[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ], 12)
        );
        assert_encode::<Bytes>(Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF]),
        (&[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ], 12)
        );
        assert_encode::<[u8; 5]>([0x00, 0x01, 0x02, 0x03, 0xFF],
        (&[
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ], 8)
        );
    }

    #[rustfmt::skip]
    #[test]
    fn sequence() {
        assert_encode::<Vec<i32>>(vec![42, 42],
            (&[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ], 12)
        );
        assert_encode::<[i32; 2]>([42, 42],
            (&[
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ], 8)
        );
        assert_encode::<&[i32]>(&[42, 42],
            (&[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ], 12)
        );
    }

    #[rustfmt::skip]
    #[test]
    fn other() {
        assert_encode::<Option<i32>>(Some(42),
            (&[
                0x00, 0x00, 0x00, 0x01, // enum tag (Some(_))
                0x00, 0x00, 0x00, 0x2A, // value
            ], 8)
        );
        assert_encode::<Option<i32>>(None,
            (&[
                0x00, 0x00, 0x00, 0x00, // enum tag (None)
            ], 4)
        );

        assert_encode::<String>(String::from("123"),
            (&[
                0x00, 0x00, 0x00, 0x03, // size
                0x31, 0x32, 0x33,       // "123"
                0x00,                   // padding
            ], 8)
        );

        assert_encode::<Cow<'_, str>>(Cow::Owned(String::from("123")),
            (&[
                0x00, 0x00, 0x00, 0x03, // size
                0x31, 0x32, 0x33,       // "123"
                0x00,                   // padding
            ], 8)
        );
        assert_encode::<Cow<'_, [u8]>>(Cow::Owned(vec![0x00, 0x01, 0x02, 0x03, 0xFF]),
            (&[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ], 12)
        );
        assert_encode::<Cow<'_, [i32]>>(Cow::Owned(vec![42, 42]),
            (&[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ], 12)
        );

        assert_encode::<Box<i32>>(Box::new(42),
            (&[
                0x00, 0x00, 0x00, 0x2A, // value
            ], 4)
        );

        assert_encode::<PhantomData<i32>>(PhantomData::default(), (&[], 0));
        assert_encode::<()>((), (&[], 0));
    }
}
