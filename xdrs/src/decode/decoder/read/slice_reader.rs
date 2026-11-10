use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use super::Read;
use crate::{Error, decode::Reference, private::Sealed};

pub struct SliceReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'r> Read<'r> for SliceReader<'r> {
    fn read_bytes<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'r, 's, [u8]>, Error> {
        let begin = self.pos;
        let end = self
            .pos
            .checked_add(len)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| {
                Error::io(IoError::new(
                    IoErrorKind::UnexpectedEof,
                    format!(
                        "the source buffer has fewer bytes remaining than \
                         requested; remaining: {remaining}, requested: {requested}",
                        remaining = self.data.len() - begin,
                        requested = len,
                    ),
                ))
            })?;

        // SAFETY: check `end <= self.data.len()` above
        let buf = unsafe { self.data.get_unchecked(begin..end) };

        self.pos = end;

        Ok(Reference::Borrowed(buf))
    }
}

impl Sealed for SliceReader<'_> {}
