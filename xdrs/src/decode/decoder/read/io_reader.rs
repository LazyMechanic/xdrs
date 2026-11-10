use super::Read;
use crate::{Error, decode::Reference, private::Sealed};

pub struct IoReader<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R> IoReader<R>
where
    R: std::io::Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
        }
    }
}

impl<'r, R> Read<'r> for IoReader<R>
where
    R: std::io::Read,
{
    #[doc(hidden)]
    fn read_bytes<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'r, 's, [u8]>, Error> {
        self.buf.resize(len, 0);
        self.reader.read_exact(&mut self.buf).map_err(Error::io)?;
        Ok(Reference::Copied(&self.buf[..len]))
    }
}

impl<R> Sealed for IoReader<R> {}
