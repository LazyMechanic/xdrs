use std::{borrow::Cow, marker::PhantomData};

use bytes::Bytes;

use crate::{
    EncodeError, ExpectedSize,
    encode::{Encode, encoder::Encoder},
};

macro_rules! impl_num {
    ($($ty:ty => $f:ident),*$(,)?) => {
        $(
        impl Encode for $ty {
            #[inline]
            fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
                encoder.$f(*self)
            }
        }
        )*
    };
}

impl_num!(
    i32 => encode_int,
    u32 => encode_uint,
    i64 => encode_hyper,
    u64 => encode_uhyper,
    f32 => encode_f32,
    f64 => encode_f64,
    bool => encode_bool,
);

impl Encode for [u8] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.encode_opaque_var(self)
    }
}

impl Encode for &'_ [u8] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.encode_opaque_var(self)
    }
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.as_slice().encode(encoder)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.encode_opaque_fix(self)
    }
}

impl Encode for str {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.encode_string(self)
    }
}

impl Encode for &'_ str {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        (*self).encode(encoder)
    }
}

impl Encode for String {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.as_str().encode(encoder)
    }
}

impl<T: Encode> Encode for [T] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        check_size(self.len(), ExpectedSize::Max(u32::MAX as usize))?;

        // Write size
        (self.len() as u32).encode(encoder)?;

        // Write data
        for elem in self {
            elem.encode(encoder)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for &'_ [T] {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        (*self).encode(encoder)
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        check_size(self.len(), ExpectedSize::Max(u32::MAX as usize))?;

        // Write data
        for elem in self {
            elem.encode(encoder)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for Vec<T> {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.as_slice().encode(encoder)
    }
}

impl Encode for Bytes {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.as_ref().encode(encoder)
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        match self {
            None => false.encode(encoder),
            Some(v) => {
                true.encode(encoder)?;
                v.encode(encoder)?;
                Ok(())
            }
        }
    }
}

impl<T: Encode> Encode for Box<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        (**self).encode(encoder)
    }
}

impl<'a, T> Encode for Cow<'a, T>
where
    T: ?Sized + ToOwned,
    T: Encode,
    T::Owned: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        (**self).encode(encoder)
    }
}

impl Encode for () {
    #[inline]
    fn encode<E: Encoder>(&self, _encoder: &mut E) -> Result<(), E::Error> {
        Ok(())
    }
}

impl<T> Encode for PhantomData<T> {
    #[inline]
    fn encode<E: Encoder>(&self, _encoder: &mut E) -> Result<(), E::Error> {
        Ok(())
    }
}

#[inline]
pub(crate) fn check_size<E: EncodeError>(
    len: usize,
    explen: ExpectedSize,
) -> Result<(), E> {
    match explen {
        ExpectedSize::Exact(e) if len != e => Err(E::invalid_size(len, explen)),
        ExpectedSize::Max(e) if len > e => Err(E::invalid_size(len, explen)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_encode;

    #[rustfmt::skip]
    #[test]
    fn numbers() {
        assert_encode::<i32>(42, &[0x00, 0x00, 0x00, 0x2A]);
        assert_encode::<i32>(-42, &[0xFF, 0xFF, 0xFF, 0xD6]);
        assert_encode::<u32>(42, &[0x00, 0x00, 0x00, 0x2A]);

        assert_encode::<i64>(42, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A]);
        assert_encode::<i64>(-42, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD6]);
        assert_encode::<u64>(42, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A]);

        assert_encode::<f32>(42.42, &[0x42, 0x29, 0xAE, 0x14]);
        assert_encode::<f64>(42.42, &[0x40, 0x45, 0x35, 0xC2, 0x8F, 0x5C, 0x28, 0xF6]);

        assert_encode::<bool>(false, &[0x00, 0x00, 0x00, 0x00]);
        assert_encode::<bool>(true, &[0x00, 0x00, 0x00, 0x01]);
    }

    #[rustfmt::skip]
    #[test]
    fn opaque() {
        assert_encode::<&[u8]>(&[0x00, 0x01, 0x02, 0x03, 0xFF],
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<Vec<u8>>(vec![0x00, 0x01, 0x02, 0x03, 0xFF],
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<Bytes>(Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF]),
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<[u8; 5]>([0x00, 0x01, 0x02, 0x03, 0xFF],
            &[
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
    }

    #[rustfmt::skip]
    #[test]
    fn sequence() {
        assert_encode::<Vec<i32>>(vec![42, 42],
            &[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ]
        );
        assert_encode::<[i32; 2]>([42, 42],
            &[
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ]
        );
        assert_encode::<&[i32]>(&[42, 42],
            &[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ]
        );
    }

    #[rustfmt::skip]
    #[test]
    fn other() {
        assert_encode::<Option<i32>>(Some(42),
            &[
                0x00, 0x00, 0x00, 0x01, // enum tag (Some(_))
                0x00, 0x00, 0x00, 0x2A, // value
            ]
        );
        assert_encode::<Option<i32>>(None,
            &[
                0x00, 0x00, 0x00, 0x00, // enum tag (None)
            ]
        );

        assert_encode::<String>(String::from("123"),
            &[
                0x00, 0x00, 0x00, 0x03, // size
                0x31, 0x32, 0x33,       // "123"
                0x00,                   // padding
            ]
        );

        assert_encode::<Cow<'_, str>>(Cow::Owned(String::from("123")),
            &[
                0x00, 0x00, 0x00, 0x03, // size
                0x31, 0x32, 0x33,       // "123"
                0x00,                   // padding
            ]
        );
        assert_encode::<Cow<'_, [u8]>>(Cow::Owned(vec![0x00, 0x01, 0x02, 0x03, 0xFF]),
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<Cow<'_, [i32]>>(Cow::Owned(vec![42, 42]),
            &[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ]
        );

        assert_encode::<Box<i32>>(Box::new(42),
            &[
                0x00, 0x00, 0x00, 0x2A, // value
            ]
        );

        assert_encode::<PhantomData<i32>>(PhantomData::default(), &[]);
        assert_encode::<()>((), &[]);
    }
}
