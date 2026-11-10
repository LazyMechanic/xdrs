use std::fmt::Debug;

use bytes::Bytes;

use crate::{
    DecodeError, Decoder, Encode, Encoder, Error, ExpectedSize,
    decode::{
        Decode, Reference,
        impls::{check_size, read_seq_var},
    },
    private::Sealed,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bounded<T, const N: usize>(T);

impl<T, const N: usize> Bounded<T, N> {
    pub fn new<'d>(value: T) -> Result<Self, Error>
    where
        T: Container<'d>,
    {
        check_size::<Error>(value.len(), ExpectedSize::Max(N))?;
        Ok(Bounded(value))
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn max_len(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Bounded<T, N> {
    const _CHECK: () = {
        assert!(
            N <= u32::MAX as usize,
            "XDR variable length type size cannot exceed (2**32)-1"
        )
    };
}

impl<'d, T, const N: usize> Encode for Bounded<T, N>
where
    T: Container<'d>,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.0.encode(encoder)
    }
}

impl<'d, T, const N: usize> Decode<'d> for Bounded<T, N>
where
    T: Container<'d>,
{
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        Ok(Self(Container::decode::<_, N>(decoder)?))
    }
}

pub trait Container<'d>: Encode + Sealed + Sized {
    #[doc(hidden)]
    fn len(&self) -> usize;

    #[doc(hidden)]
    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error>;
}

impl<'d, T: Decode<'d> + Encode> Container<'d> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        Ok(read_seq_var(decoder, Some(N))?)
    }
}

impl<'d: 'a, 'a> Container<'d> for &'a [u8] {
    fn len(&self) -> usize {
        (*self).len()
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let v = decoder.decode_opaque_var(Some(N))?;
        match v {
            Reference::Borrowed(v) => {
                check_size(v.len(), ExpectedSize::Max(N))?;
                Ok(v)
            }
            Reference::Copied(_) => Err(D::Error::expected_borrowed()),
        }
    }
}

impl<'d> Container<'d> for Vec<u8> {
    fn len(&self) -> usize {
        Vec::<u8>::len(self)
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let v = decoder.decode_opaque_var(Some(N))?;
        let v = v.as_ref();
        check_size(v.len(), ExpectedSize::Max(N))?;
        Ok(Vec::from(v))
    }
}

impl<'d> Container<'d> for Bytes {
    fn len(&self) -> usize {
        Bytes::len(self)
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let v = decoder.decode_opaque_var(Some(N))?;
        let v = v.as_ref();
        check_size(v.len(), ExpectedSize::Max(N))?;
        Ok(Bytes::copy_from_slice(v))
    }
}

impl<'d: 'a, 'a> Container<'d> for &'a str {
    fn len(&self) -> usize {
        str::len(self)
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let v = decoder.decode_string(Some(N))?;
        match v {
            Reference::Borrowed(v) => {
                check_size(v.len(), ExpectedSize::Max(N))?;
                Ok(v)
            }
            Reference::Copied(_) => Err(D::Error::expected_borrowed()),
        }
    }
}

impl<'d> Container<'d> for String {
    fn len(&self) -> usize {
        String::len(self)
    }

    fn decode<D: Decoder<'d>, const N: usize>(
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let v = decoder.decode_string(Some(N))?;
        let v = v.as_ref();
        check_size(v.len(), ExpectedSize::Max(N))?;
        Ok(String::from(v))
    }
}

impl<T> Sealed for Vec<T> {}
impl Sealed for &'_ [u8] {}
impl Sealed for Bytes {}
impl Sealed for &'_ str {}
impl Sealed for String {}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    use crate::{
        StdDecoder,
        decode::SliceReader,
        test_utils::{assert_decode, assert_encode},
    };

    #[track_caller]
    pub fn assert_decode_err<'d, T: Decode<'d> + PartialEq + Debug>(
        val: &'d [u8],
    ) {
        let reader = SliceReader::new(val);
        let mut decoder = StdDecoder::new(reader);

        let _ = T::decode(&mut decoder).unwrap_err();
    }

    #[test]
    fn new_ok() {
        let _ = Bounded::<_, 5>::new(vec![0i32; 5]).unwrap();

        let _ = Bounded::<_, 5>::new(vec![0u8; 5]).unwrap();
        let _ = Bounded::<_, 5>::new(&[0u8, 0u8, 0u8, 0u8, 0u8][..]).unwrap();
        let _ = Bounded::<_, 5>::new(Bytes::from_static(&[
            0u8, 0u8, 0u8, 0u8, 0u8,
        ]))
        .unwrap();

        let _ = Bounded::<_, 5>::new("12345").unwrap();
        let _ = Bounded::<_, 5>::new(String::from("12345")).unwrap();
    }

    #[test]
    fn new_err() {
        let _ = Bounded::<_, 5>::new(vec![0i32; 6]).unwrap_err();

        let _ = Bounded::<_, 5>::new(vec![0u8; 6]).unwrap_err();
        let _ = Bounded::<_, 5>::new(&[0u8, 0u8, 0u8, 0u8, 0u8, 0u8][..])
            .unwrap_err();
        let _ = Bounded::<_, 5>::new(Bytes::from_static(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]))
        .unwrap_err();

        let _ = Bounded::<_, 5>::new("123456").unwrap_err();
        let _ = Bounded::<_, 5>::new(String::from("123456")).unwrap_err();
    }

    #[rustfmt::skip]
    #[test]
    fn decode_opaque_ok() {
        assert_decode::<Bounded<Vec<u8>, 5>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], Bounded(vec![0x00, 0x01, 0x02, 0x03, 0xFF]));
        assert_decode::<Bounded<Bytes, 5>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], Bounded(Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF])));
        assert_decode::<Bounded<&[u8], 5>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], Bounded(&[0x00, 0x01, 0x02, 0x03, 0xFF]));
    }

    #[rustfmt::skip]
    #[test]
    fn decode_opaque_err() {
        assert_decode_err::<Bounded<Vec<u8>, 4>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ]);
        assert_decode_err::<Bounded<Bytes, 4>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ]);
        assert_decode_err::<Bounded<&[u8], 4>>(&[
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ]);
    }

    #[rustfmt::skip]
    #[test]
    fn decode_sequence_ok() {
        assert_decode::<Bounded<Vec<i32>, 2>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], Bounded(vec![42, 42]));
    }

    #[rustfmt::skip]
    #[test]
    fn decode_sequence_err() {
        assert_decode_err::<Bounded<Vec<i32>, 1>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ]);
    }

    #[rustfmt::skip]
    #[test]
    fn encode_opaque_ok() {
        assert_encode::<Bounded<Vec<u8>, 5>>(
            Bounded(vec![0x00, 0x01, 0x02, 0x03, 0xFF]),
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<Bounded<Bytes, 5>>(
            Bounded(Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF])),
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
        assert_encode::<Bounded<&[u8], 5>>(
            Bounded(&[0x00, 0x01, 0x02, 0x03, 0xFF]),
            &[
                0x00, 0x00, 0x00, 0x05,       // size
                0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
                0x00, 0x00, 0x00,             // padding
            ]
        );
    }

    #[rustfmt::skip]
    #[test]
    fn encode_sequence_ok() {
        assert_encode::<Bounded<Vec<i32>, 2>>(
            Bounded(vec![42, 42]),
            &[
                0x00, 0x00, 0x00, 0x02, // size
                0x00, 0x00, 0x00, 0x2A, // [0]
                0x00, 0x00, 0x00, 0x2A, // [1]
            ]
        );
    }
}
