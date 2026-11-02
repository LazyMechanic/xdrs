mod impls;

pub mod read;

use read::Read;

use crate::error::Error;

pub trait Decode<'a>: Sized {
    fn decode<R: Read<'a>>(reader: &mut R) -> Result<(Self, usize), Error>;
}
