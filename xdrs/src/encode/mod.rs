mod encoder;
pub(crate) mod impls;

pub use self::encoder::*;
use crate::Error;

pub trait Encode {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error>;
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: std::io::Write,
    T: Encode,
{
    let mut encoder = StdEncoder::new(writer);
    value.encode(&mut encoder)
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Encode,
{
    let mut encoder = StdEncoder::new(Vec::new());
    value.encode(&mut encoder)?;
    Ok(encoder.into_inner())
}
