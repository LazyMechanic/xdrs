mod read;
mod reference;
mod std_decoder;

use std::fmt::Display;

pub use self::{
    read::{IoReader, Read, SliceReader},
    reference::Reference,
    std_decoder::StdDecoder,
};
use crate::ExpectedSize;

pub trait DecodeError: std::error::Error + Sized {
    fn custom<T: Display>(msg: T) -> Self;

    fn invalid_size(len: usize, explen: ExpectedSize) -> Self {
        Self::custom(format_args!(
            "invalid size: {len}, expected: {}",
            explen.display()
        ))
    }

    fn expected_borrowed() -> Self {
        Self::custom("expected borrowed value")
    }

    fn unknown_variant(variant: i32) -> Self {
        Self::custom(format_args!("unknown variant: {variant}"))
    }
}

pub trait Decoder<'d> {
    type Error: DecodeError;

    fn decode_int(&mut self) -> Result<i32, Self::Error>;
    fn decode_uint(&mut self) -> Result<u32, Self::Error>;
    fn decode_hyper(&mut self) -> Result<i64, Self::Error>;
    fn decode_uhyper(&mut self) -> Result<u64, Self::Error>;
    fn decode_f32(&mut self) -> Result<f32, Self::Error>;
    fn decode_f64(&mut self) -> Result<f64, Self::Error>;
    fn decode_bool(&mut self) -> Result<bool, Self::Error>;

    fn decode_opaque_fix<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'d, 's, [u8]>, Self::Error>
    where
        'd: 's;
    fn decode_opaque_var<'s>(
        &'s mut self,
        maxlen: Option<usize>,
    ) -> Result<Reference<'d, 's, [u8]>, Self::Error>
    where
        'd: 's;

    fn decode_string<'s>(
        &'s mut self,
        maxlen: Option<usize>,
    ) -> Result<Reference<'d, 's, str>, Self::Error>
    where
        'd: 's;
}
