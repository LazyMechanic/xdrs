use std::io::Write;

use bytes::Bytes;

use crate::{
    Encode,
    decode::{Decode, read::Read},
    encode::impls::check_size_var,
    error::Error,
    private::{Sealed, impl_sealed},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Limited<T: Container, const N: usize>(T);

impl<T, const N: usize> Limited<T, N>
where
    T: Container,
{
    const _CHECK: () = {
        assert!(
            N <= u32::MAX as usize,
            "XDR variable length type size cannot exceed (2**32)-1"
        )
    };
}

impl<T, const N: usize> Encode for Limited<T, N>
where
    T: Container + Encode,
{
    fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
        self.0.encode(out)
    }
}

impl<'a, T, const N: usize> Decode<'a> for Limited<T, N>
where
    T: Container + Decode<'a>,
{
    fn decode<R: Read<'a>>(reader: &mut R) -> Result<(Self, usize), Error> {
        let (v, n) = T::decode(reader)?;
        check_size_var(v.len(), N)?;
        Ok((Self(v), n))
    }
}

pub trait Container: Sealed {
    fn len(&self) -> usize;
}

impl<T> Container for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Container for Bytes {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Container for String {
    fn len(&self) -> usize {
        self.len()
    }
}

impl_sealed!(String, Bytes);
impl<T> Sealed for Vec<T> {}
