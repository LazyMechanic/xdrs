mod decoder;
pub(crate) mod impls;

pub use self::decoder::*;
use crate::Error;

pub trait Decode<'d>: Sized {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error>;
}

pub trait DecodeOwned: for<'d> Decode<'d> {}
impl<T> DecodeOwned for T where T: for<'d> Decode<'d> {}

pub trait DecodeDiscriminant<'d>: Decode<'d> {
    fn decode_discriminant<D: Decoder<'d>>(
        decoder: &mut D,
    ) -> Result<(Option<Self>, i32), D::Error>;
}

pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: std::io::Read,
    T: DecodeOwned,
{
    let mut decoder = StdDecoder::new(IoReader::new(reader));
    T::decode(&mut decoder)
}

pub fn from_slice<'d, T>(slice: &'d [u8]) -> Result<T, Error>
where
    T: Decode<'d>,
{
    let mut decoder = StdDecoder::new(SliceReader::new(slice));
    T::decode(&mut decoder)
}
