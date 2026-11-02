pub(crate) mod impls;

use std::io::Write;

use crate::error::Error;

pub trait Encode {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error>;
}
