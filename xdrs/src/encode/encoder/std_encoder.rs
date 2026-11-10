use std::io::Write;

use crate::{
    Error, ExpectedSize,
    encode::{encoder::Encoder, impls::check_size},
    padding,
};

pub struct StdEncoder<W> {
    writer: W,
}

impl<W> StdEncoder<W>
where
    W: Write,
{
    /// Creates a new XDR encoder.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Unwrap the `Writer` from the `Encoder`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W> Encoder for StdEncoder<W>
where
    W: Write,
{
    type Error = Error;

    fn encode_int(&mut self, value: i32) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_uint(&mut self, value: u32) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_hyper(&mut self, value: i64) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_uhyper(&mut self, value: u64) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_f32(&mut self, value: f32) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_f64(&mut self, value: f64) -> Result<(), Self::Error> {
        self.writer
            .write_all(&value.to_be_bytes())
            .map_err(Error::io)
    }

    fn encode_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        if value {
            self.encode_int(1)
        } else {
            self.encode_int(0)
        }
    }

    fn encode_opaque_fix(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        check_size::<Self::Error>(
            value.len(),
            ExpectedSize::Max(u32::MAX as usize),
        )?;

        // Write data
        self.writer.write_all(value).map_err(Error::io)?;

        // Write padding
        let padding = padding(value.len());
        if !padding.is_empty() {
            self.writer.write_all(&padding).map_err(Error::io)?;
        }

        Ok(())
    }

    fn encode_opaque_var(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        check_size::<Self::Error>(
            value.len(),
            ExpectedSize::Max(u32::MAX as usize),
        )?;

        // Write size
        self.encode_uint(value.len() as u32)?;

        // Write data
        self.writer.write_all(value).map_err(Error::io)?;

        // Write padding
        let padding = padding(value.len());
        if !padding.is_empty() {
            self.writer.write_all(&padding).map_err(Error::io)?;
        }

        Ok(())
    }

    fn encode_string(&mut self, value: &str) -> Result<(), Self::Error> {
        self.encode_opaque_var(value.as_bytes())
    }
}
