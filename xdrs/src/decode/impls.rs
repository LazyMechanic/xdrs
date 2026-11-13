use std::{borrow::Cow, marker::PhantomData, mem::MaybeUninit};

use bytes::Bytes;

use crate::{
    ExpectedSize,
    decode::{
        Decode, DecodeDiscriminant, Reference,
        decoder::{DecodeError, Decoder},
    },
};

macro_rules! impl_primitive {
    ($($ty:ty => $f:ident),*$(,)?) => {
        $(
        impl<'a> Decode<'a> for $ty {
            #[inline]
            fn decode<D: Decoder<'a>>(decoder: &mut D) -> Result<Self, D::Error> {
                decoder.$f()
            }
        }
        )*
    }
}

impl_primitive!(
    i32 => decode_int,
    u32 => decode_uint,
    i64 => decode_hyper,
    u64 => decode_uhyper,
    f32 => decode_f32,
    f64 => decode_f64,
    bool => decode_bool,
);

impl<'d> DecodeDiscriminant<'d> for i32 {
    fn decode_discriminant<D: Decoder<'d>>(
        decoder: &mut D,
    ) -> Result<(Option<Self>, i32), D::Error> {
        let discr_orig = i32::decode(decoder)?;
        Ok((Some(discr_orig), discr_orig))
    }
}

impl<'d> DecodeDiscriminant<'d> for u32 {
    fn decode_discriminant<D: Decoder<'d>>(
        decoder: &mut D,
    ) -> Result<(Option<Self>, i32), D::Error> {
        let discr_i32 = i32::decode(decoder)?;
        Ok((Some(discr_i32 as _), discr_i32))
    }
}

impl<'d> DecodeDiscriminant<'d> for bool {
    fn decode_discriminant<D: Decoder<'d>>(
        decoder: &mut D,
    ) -> Result<(Option<Self>, i32), D::Error> {
        match decoder.decode_int()? {
            0 => Ok((Some(false), 0)),
            1 => Ok((Some(true), 1)),
            other => Ok((None, other)),
        }
    }
}

impl<'d: 'a, 'a> Decode<'d> for &'a [u8] {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        match decoder.decode_opaque_var(None)? {
            Reference::Borrowed(v) => Ok(v),
            Reference::Copied(_) => Err(D::Error::expected_borrowed()),
        }
    }
}

impl<'d> Decode<'d> for Vec<u8> {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        decoder
            .decode_opaque_var(None)
            .map(|v| Vec::from(v.as_ref()))
    }
}

impl<'d, const N: usize> Decode<'d> for [u8; N] {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        let slice = decoder.decode_opaque_fix(N)?;
        let slice = slice.as_ref();
        check_size(slice.len(), ExpectedSize::Exact(N))?;

        let mut arr = [0; N];
        // SAFETY: decode_opaque_fix(N) returns exactly N bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), arr.as_mut_ptr(), N)
        };

        Ok(arr)
    }
}

impl<'d: 'a, 'a> Decode<'d> for &'a str {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        match decoder.decode_string(None)? {
            Reference::Borrowed(v) => Ok(v),
            Reference::Copied(_) => Err(D::Error::expected_borrowed()),
        }
    }
}

impl<'d> Decode<'d> for String {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        decoder
            .decode_string(None)
            .map(|v| String::from(v.as_ref()))
    }
}

impl<'d, T: Decode<'d>, const N: usize> Decode<'d> for [T; N] {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        read_seq_fix(decoder)
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Vec<T> {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        read_seq_var(decoder, None)
    }
}

impl<'d> Decode<'d> for Bytes {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        Vec::<u8>::decode(decoder).map(Bytes::from)
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Option<T> {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        let have = bool::decode(decoder)?;
        if have {
            Ok(Some(T::decode(decoder)?))
        } else {
            Ok(None)
        }
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Box<T> {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        T::decode(decoder).map(Box::new)
    }
}

impl<'d, 'a, T> Decode<'d> for Cow<'a, T>
where
    T: ?Sized + ToOwned,
    T::Owned: Decode<'d>,
{
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        T::Owned::decode(decoder).map(Cow::Owned)
    }
}

impl<'a> Decode<'a> for () {
    #[inline]
    fn decode<D: Decoder<'a>>(_decoder: &mut D) -> Result<Self, D::Error> {
        Ok(())
    }
}

impl<'a, T> Decode<'a> for PhantomData<T> {
    #[inline]
    fn decode<D: Decoder<'a>>(_decoder: &mut D) -> Result<Self, D::Error> {
        Ok(Default::default())
    }
}

fn read_seq_fix<'d, D, T, const N: usize>(
    decoder: &mut D,
) -> Result<[T; N], D::Error>
where
    D: Decoder<'d>,
    T: Decode<'d>,
{
    const {
        assert!(
            N <= u32::MAX as usize,
            "XDR fix length type size cannot exceed (2**32)-1"
        );
    };

    struct Guard<'a, I> {
        array_mut: &'a mut [MaybeUninit<I>],
        initialized: usize,
    }
    impl<I> Guard<'_, I> {
        #[inline]
        pub unsafe fn push_unchecked(&mut self, item: I) {
            // SAFETY: If `initialized` was correct before and the caller does not
            // invoke this method more than N times, then writes will be in-bounds
            // and slots will not be initialized more than once.
            unsafe {
                self.array_mut
                    .get_unchecked_mut(self.initialized)
                    .write(item);
                self.initialized = self.initialized.unchecked_add(1);
            }
        }
    }
    impl<I> Drop for Guard<'_, I> {
        fn drop(&mut self) {
            debug_assert!(self.initialized <= self.array_mut.len());

            // SAFETY: this slice will contain only initialized objects.
            if !self.array_mut.is_empty() {
                // SAFETY: the caller must guarantee that every element of `self`
                // is initialized and satisfies all invariants of `I`.
                // Dropping the value in place is safe if that is the case.
                unsafe {
                    std::ptr::drop_in_place(
                        self.array_mut as *mut [MaybeUninit<I>] as *mut [I],
                    )
                }
            }
        }
    }

    let mut arr = [const { MaybeUninit::uninit() }; N];

    let mut guard = Guard {
        array_mut: &mut arr,
        initialized: 0,
    };
    while guard.initialized < guard.array_mut.len() {
        let item = T::decode(decoder)?;

        // SAFETY: The loop condition ensures we have space to push the item
        unsafe { guard.push_unchecked(item) };
    }

    std::mem::forget(guard);

    // SAFETY: Array fully initialized above
    let arr = unsafe {
        let ptr = &arr as *const [MaybeUninit<T>; N] as *const [T; N];
        std::ptr::read(ptr)
    };

    Ok(arr)
}

pub fn read_seq_var<'d, D, T>(
    decoder: &mut D,
    maxlen: Option<usize>,
) -> Result<Vec<T>, D::Error>
where
    D: Decoder<'d>,
    T: Decode<'d>,
{
    const MAX_RESERVED_SIZE: usize = 1024; // 1024 elements

    let len = decoder.decode_uint().map(|v| v as usize)?;
    if let Some(maxlen) = maxlen {
        check_size(len, ExpectedSize::Max(maxlen))?
    }

    let mut vec = Vec::with_capacity(usize::min(len, MAX_RESERVED_SIZE));
    for _ in 0..len {
        let item = T::decode(decoder)?;

        vec.push(item);
    }

    Ok(vec)
}

#[inline]
pub(crate) fn check_size<E: DecodeError>(
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
    use crate::test_utils::{assert_decode, assert_decode_discriminant};

    #[rustfmt::skip]
    #[test]
    fn decode_discriminant() {
        assert_decode_discriminant::<i32>(&(-1i32).to_be_bytes(), (Some(-1i32), -1i32));
        assert_decode_discriminant::<i32>(&2i32.to_be_bytes(), (Some(2i32), 2i32));
        assert_decode_discriminant::<i32>(&3i32.to_be_bytes(), (Some(3i32), 3i32));
        assert_decode_discriminant::<i32>(&251252114i32.to_be_bytes(), (Some(251252114i32), 251252114i32));
        assert_decode_discriminant::<i32>(&i32::MAX.to_be_bytes(), (Some(i32::MAX), i32::MAX));
        assert_decode_discriminant::<i32>(&i32::MIN.to_be_bytes(), (Some(i32::MIN), i32::MIN));

        assert_decode_discriminant::<u32>(&(-1i32).to_be_bytes(), (Some(4294967295), -1i32));
        assert_decode_discriminant::<u32>(&2i32.to_be_bytes(), (Some(2u32), 2i32));
        assert_decode_discriminant::<u32>(&3i32.to_be_bytes(), (Some(3u32), 3i32));
        assert_decode_discriminant::<u32>(&251252114i32.to_be_bytes(), (Some(251252114u32), 251252114i32));
        assert_decode_discriminant::<u32>(&i32::MAX.to_be_bytes(), (Some(2147483647u32), i32::MAX));
        assert_decode_discriminant::<u32>(&i32::MIN.to_be_bytes(), (Some(2147483648u32), i32::MIN));

        assert_decode_discriminant::<bool>(&1i32.to_be_bytes(), (Some(true), 1i32));
        assert_decode_discriminant::<bool>(&0i32.to_be_bytes(), (Some(false), 0i32));
        assert_decode_discriminant::<bool>(&i32::MAX.to_be_bytes(), (None, i32::MAX));
    }

    #[rustfmt::skip]
    #[test]
    fn numbers() {
        assert_decode::<i32>(&[0x00, 0x00, 0x00, 0x2A], 42);
        assert_decode::<i32>(&[0xFF, 0xFF, 0xFF, 0xD6], -42);
        assert_decode::<u32>(&[0x00, 0x00, 0x00, 0x2A], 42);

        assert_decode::<i64>(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], 42);
        assert_decode::<i64>(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD6], -42);
        assert_decode::<u64>(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], 42);

        assert_decode::<f32>(&[0x42, 0x29, 0xAE, 0x14], 42.42);
        assert_decode::<f64>(&[0x40, 0x45, 0x35, 0xC2, 0x8F, 0x5C, 0x28, 0xF6], 42.42);

        assert_decode::<bool>(&[0x00, 0x00, 0x00, 0x00], false);
        assert_decode::<bool>(&[0x00, 0x00, 0x00, 0x01], true);
    }

    #[rustfmt::skip]
    #[test]
    fn opaque() {
        assert_decode::<Vec<u8>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], vec![0x00, 0x01, 0x02, 0x03, 0xFF]);
        assert_decode::<Bytes>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF]));
        assert_decode::<[u8; 5]>(&[
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], [0x00, 0x01, 0x02, 0x03, 0xFF]);
    }

    #[rustfmt::skip]
    #[test]
    fn sequence() {
        assert_decode::<Vec<i32>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], vec![42, 42]);
        assert_decode::<[i32; 2]>(&[
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], [42, 42]);
    }

    #[rustfmt::skip]
    #[test]
    fn other() {
        assert_decode::<Option<i32>>(&[
            0x00, 0x00, 0x00, 0x01, // enum tag (Some(_))
            0x00, 0x00, 0x00, 0x2A, // value
        ], Some(42));
        assert_decode::<Option<i32>>(&[
            0x00, 0x00, 0x00, 0x00, // enum tag (None)
        ], None);

        assert_decode::<String>(&[
            0x00, 0x00, 0x00, 0x03, // size
            0x31, 0x32, 0x33,       // "123"
            0x00,                   // padding
        ], String::from("123"));

        assert_decode::<Cow<'_, str>>(&[
            0x00, 0x00, 0x00, 0x03, // size
            0x31, 0x32, 0x33,       // "123"
            0x00,                   // padding
        ], Cow::Owned(String::from("123")));
        assert_decode::<Cow<'_, [u8]>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], Cow::Owned(vec![0x00, 0x01, 0x02, 0x03, 0xFF]));
        assert_decode::<Cow<'_, [i32]>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], Cow::Owned(vec![42, 42]));

        assert_decode::<Box<i32>>(&[
            0x00, 0x00, 0x00, 0x2A, // value
        ], Box::new(42));

        assert_decode::<PhantomData<i32>>(&[], PhantomData::default());
        assert_decode::<()>(&[], ());
    }
}
