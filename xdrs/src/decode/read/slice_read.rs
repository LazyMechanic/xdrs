use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use crate::{Error, PADDING_MAX_LEN, Read};

pub struct SliceRead<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SliceRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    fn get_next(&mut self, requested: usize) -> Result<&'a [u8], Error> {
        let begin = self.pos;
        let end = self
            .pos
            .checked_add(requested)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| {
                Error::io(IoError::new(
                    IoErrorKind::UnexpectedEof,
                    format!(
                        "the source buffer has fewer bytes remaining than \
                         requested; remaining: {remaining}, requested: {requested}",
                        remaining = self.data.len() - begin
                    ),
                ))
            })?;

        // SAFETY: check `end <= self.data.len()` above
        let buf = unsafe { self.data.get_unchecked(begin..end) };

        self.pos = end;

        Ok(buf)
    }
}

impl<'a> Read<'a> for SliceRead<'a> {
    fn read_bytes(&mut self, len: usize) -> Result<(&'a [u8], usize), Error> {
        assert!(
            len <= u32::MAX as usize,
            "maximum of bytes length is (2**32)-1"
        );

        let padding_len =
            (PADDING_MAX_LEN - (len % PADDING_MAX_LEN)) % PADDING_MAX_LEN;
        // If no padding just read next block
        if padding_len == 0 {
            let slice = self.get_next(len)?;
            return Ok((slice, len));
        }

        let len_with_padding = {
            // SAFETY: already check that len can't be bigger than u32::MAX
            let len = len as u32;
            // SAFETY: padding_len <= PADDING_MAX_LEN
            let padding_len = padding_len as u32;
            len.checked_add(padding_len).ok_or_else(|| {
                Error::io(IoError::new(
                    IoErrorKind::OutOfMemory,
                    format!(
                        "data length, including padding, exceeds the maximum allowable limit, \
                         len: {len}, padding_len: {padding_len}, remaining: {remaining}",
                        remaining = self.data.len()
                    ),
                ))
            }).map(|v| {
                // SAFETY: size_of::<i32>() <= size_of::<usize>()
                v as usize
            })?
        };

        let slice_with_padding = self.get_next(len_with_padding)?;
        // SAFETY: self.get_next(len_with_padding) returns exactly len_with_padding bytes.
        let payload = unsafe { slice_with_padding.get_unchecked(..len) };

        Ok((payload, len_with_padding))
    }
}
