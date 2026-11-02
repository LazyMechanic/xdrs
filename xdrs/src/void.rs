use std::io::Write;

use crate::{Decode, Encode, Error, Read};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Void;

impl Encode for Void {
    fn encode<W: Write>(&self, _writer: &mut W) -> Result<usize, Error> {
        Ok(0)
    }
}

impl<'a> Decode<'a> for Void {
    fn decode<R: Read<'a>>(_reader: &mut R) -> Result<(Self, usize), Error> {
        Ok((Void, 0))
    }
}
