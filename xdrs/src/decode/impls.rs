use std::{borrow::Cow, marker::PhantomData, mem::MaybeUninit};

use bytes::Bytes;

use crate::{
    decode::{Decode, read::Read},
    error::Error,
};

macro_rules! impl_primitive {
    ($($ty:ty => $f:ident),*$(,)?) => {
        $(
        impl<'a> Decode<'a> for $ty {
            #[inline]
            fn decode<R: Read<'a>>(reader: &mut R) -> Result<(Self, usize), Error> {
                reader.$f()
            }
        }
        )*
    }
}

impl_primitive!(
    i32 => read_i32,
    u32 => read_u32,
    i64 => read_i64,
    u64 => read_u64,
    f32 => read_f32,
    f64 => read_f64,
    bool => read_bool,
);

impl<'d: 'a, 'a> Decode<'d> for &'a [u8] {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let mut read = 0;

        let (sz, n) = read_size(reader)?;
        read += n;

        let (slice, n) = read_bytes_var(reader, sz)?;
        read += n;

        Ok((slice, read))
    }
}

impl<'d> Decode<'d> for Vec<u8> {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (slice, n) = <&[u8] as Decode>::decode(reader)?;
        Ok((slice.to_vec(), n))
    }
}

impl<'d, const N: usize> Decode<'d> for [u8; N] {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        read_bytes_fix(reader)
    }
}

impl<'d: 'a, 'a> Decode<'d> for &'a str {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (v, sz) = <&[u8] as Decode>::decode(reader)?;
        if !v.is_ascii() {
            return Err(Error::invalid_value(
                "not ASCII string",
                "valid ASCII string",
            ));
        }

        // SAFETY: ASCII is a subset of UTF-8. If string is valid ASCII then
        // it is valid UTF-8. This check was performed above.
        let s = unsafe { str::from_utf8_unchecked(v) };
        Ok((s, sz))
    }
}

impl<'d> Decode<'d> for String {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (s, sz) = <&str as Decode>::decode(reader)?;
        Ok((String::from(s), sz))
    }
}

impl<'d, T: Decode<'d>, const N: usize> Decode<'d> for [T; N] {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        read_seq_fix(reader)
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Vec<T> {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let mut read = 0;

        let (sz, n) = read_size(reader)?;
        read += n;

        let (vec, n) = read_seq_var(reader, sz)?;
        read += n;

        Ok((vec, read))
    }
}

impl<'d> Decode<'d> for Bytes {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (vec, read) = Vec::<u8>::decode(reader)?;
        Ok((Bytes::from(vec), read))
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Option<T> {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (have, mut sz) = bool::decode(reader)?;
        let ret = if have {
            let (v, osz) = T::decode(reader)?;
            sz += osz;
            Some(v)
        } else {
            None
        };
        Ok((ret, sz))
    }
}

impl<'d, T: Decode<'d>> Decode<'d> for Box<T> {
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (b, sz) = T::decode(reader)?;
        Ok((Box::new(b), sz))
    }
}

impl<'d, 'a, T> Decode<'d> for Cow<'a, T>
where
    T: ?Sized + ToOwned,
    T::Owned: Decode<'d>,
{
    fn decode<R: Read<'d>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (b, sz) = T::Owned::decode(reader)?;
        Ok((Cow::Owned(b), sz))
    }
}

impl<'a> Decode<'a> for () {
    #[inline]
    fn decode<R: Read<'a>>(_reader: &mut R) -> Result<(Self, usize), Error> {
        Ok(((), 0))
    }
}

impl<'a, T> Decode<'a> for PhantomData<T> {
    #[inline]
    fn decode<R: Read<'a>>(_reader: &mut R) -> Result<(Self, usize), Error> {
        Ok((Default::default(), 0))
    }
}

#[inline]
fn read_size<'d, R>(reader: &mut R) -> Result<(usize, usize), Error>
where
    R: Read<'d>,
{
    u32::decode(reader).map(|(sz, n)| (sz as usize, n))
}

fn read_seq_fix<'d, R, T, const N: usize>(
    reader: &mut R,
) -> Result<([T; N], usize), Error>
where
    R: Read<'d>,
    T: Decode<'d>,
{
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

    let mut read = 0;
    let mut arr = [const { MaybeUninit::uninit() }; N];

    let mut guard = Guard {
        array_mut: &mut arr,
        initialized: 0,
    };
    while guard.initialized < guard.array_mut.len() {
        let (item, n) = T::decode(reader)?;
        read += n;

        // SAFETY: The loop condition ensures we have space to push the item
        unsafe { guard.push_unchecked(item) };
    }
    std::mem::forget(guard);

    // SAFETY: Array fully initialized above
    let arr = unsafe {
        let ptr = &arr as *const [MaybeUninit<T>; N] as *const [T; N];
        std::ptr::read(ptr)
    };

    Ok((arr, read))
}

fn read_seq_var<'d, R, T>(
    reader: &mut R,
    sz: usize,
) -> Result<(Vec<T>, usize), Error>
where
    R: Read<'d>,
    T: Decode<'d>,
{
    const MAX_RESERVED_SIZE: usize = 1024; // 1024 elements

    let mut read = 0;

    let mut vec = Vec::with_capacity(usize::min(sz, MAX_RESERVED_SIZE));
    for _ in 0..sz {
        let (item, n) = T::decode(reader)?;
        read += n;

        vec.push(item);
    }

    Ok((vec, read))
}

#[inline]
fn read_bytes_fix<'d, R, const N: usize>(
    reader: &mut R,
) -> Result<([u8; N], usize), Error>
where
    R: Read<'d>,
{
    let (slice, n) = reader.read_bytes(N)?;

    let mut arr = [0; N];
    // SAFETY: read_bytes(N) returns exactly N bytes.
    unsafe {
        core::ptr::copy_nonoverlapping(slice.as_ptr(), arr.as_mut_ptr(), N)
    };

    Ok((arr, n))
}

#[inline]
fn read_bytes_var<'d, R>(
    reader: &mut R,
    len: usize,
) -> Result<(&'d [u8], usize), Error>
where
    R: Read<'d>,
{
    reader.read_bytes(len)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    use crate::decode::read::SliceRead;

    #[track_caller]
    fn assert_decode<'d, T: Decode<'d> + PartialEq + Debug>(
        val: &'d [u8],
        exp: (T, usize),
    ) {
        let mut reader = SliceRead::new(val);
        assert_eq!(T::decode(&mut reader).unwrap(), exp);
    }

    #[rustfmt::skip]
    #[test]
    fn numbers() {
        assert_decode::<i32>(&[0x00, 0x00, 0x00, 0x2A], (42, 4));
        assert_decode::<i32>(&[0xFF, 0xFF, 0xFF, 0xD6], (-42, 4));
        assert_decode::<u32>(&[0x00, 0x00, 0x00, 0x2A], (42, 4));

        assert_decode::<i64>(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], (42, 8));
        assert_decode::<i64>(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD6], (-42, 8));
        assert_decode::<u64>(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A], (42, 8));

        assert_decode::<f32>(&[0x42, 0x29, 0xAE, 0x14], (42.42, 4));
        assert_decode::<f64>(&[0x40, 0x45, 0x35, 0xC2, 0x8F, 0x5C, 0x28, 0xF6], (42.42, 8));

        assert_decode::<bool>(&[0x00, 0x00, 0x00, 0x00], (false, 4));
        assert_decode::<bool>(&[0x00, 0x00, 0x00, 0x01], (true, 4));
    }

    #[rustfmt::skip]
    #[test]
    fn opaque() {
        assert_decode::<Vec<u8>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], (vec![0x00, 0x01, 0x02, 0x03, 0xFF], 12));
        assert_decode::<Bytes>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], (Bytes::from_static(&[0x00, 0x01, 0x02, 0x03, 0xFF]), 12));
        assert_decode::<[u8; 5]>(&[
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], ([0x00, 0x01, 0x02, 0x03, 0xFF], 8));
    }

    #[rustfmt::skip]
    #[test]
    fn sequence() {
        assert_decode::<Vec<i32>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], (vec![42, 42], 12));
        assert_decode::<[i32; 2]>(&[
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], ([42, 42], 8));
    }

    #[rustfmt::skip]
    #[test]
    fn other() {
        assert_decode::<Option<i32>>(&[
            0x00, 0x00, 0x00, 0x01, // enum tag (Some(_))
            0x00, 0x00, 0x00, 0x2A, // value
        ], (Some(42), 8));
        assert_decode::<Option<i32>>(&[
            0x00, 0x00, 0x00, 0x00, // enum tag (None)
        ], (None, 4));

        assert_decode::<String>(&[
            0x00, 0x00, 0x00, 0x03, // size
            0x31, 0x32, 0x33,       // "123"
            0x00,                   // padding
        ], (String::from("123"), 8));

        assert_decode::<Cow<'_, str>>(&[
            0x00, 0x00, 0x00, 0x03, // size
            0x31, 0x32, 0x33,       // "123"
            0x00,                   // padding
        ], (Cow::Owned(String::from("123")), 8));
        assert_decode::<Cow<'_, [u8]>>(&[
            0x00, 0x00, 0x00, 0x05,       // size
            0x00, 0x01, 0x02, 0x03, 0xFF, // [0..5]
            0x00, 0x00, 0x00,             // padding
        ], (Cow::Owned(vec![0x00, 0x01, 0x02, 0x03, 0xFF]), 12));
        assert_decode::<Cow<'_, [i32]>>(&[
            0x00, 0x00, 0x00, 0x02, // size
            0x00, 0x00, 0x00, 0x2A, // [0]
            0x00, 0x00, 0x00, 0x2A, // [1]
        ], (Cow::Owned(vec![42, 42]), 12));

        assert_decode::<Box<i32>>(&[
            0x00, 0x00, 0x00, 0x2A, // value
        ], (Box::new(42), 4));

        assert_decode::<PhantomData<i32>>(&[], (PhantomData::default(), 0));
        assert_decode::<()>(&[], ((), 0));
    }
}
