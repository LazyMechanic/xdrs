mod std_encoder;

use std::fmt::Display;

pub use std_encoder::StdEncoder;

use crate::ExpectedSize;

pub trait EncodeError: std::error::Error + Sized {
    fn custom<T: Display>(msg: T) -> Self;

    fn invalid_size(len: usize, explen: ExpectedSize) -> Self {
        Self::custom(format_args!(
            "invalid size: {}, expected: {}",
            len,
            explen.display()
        ))
    }

    fn invalid_variant<T: Display>(reason: T) -> Self {
        Self::custom(format_args!("invalid variant: {reason}"))
    }
}

pub trait Encoder {
    type Error: EncodeError;

    fn encode_int(&mut self, value: i32) -> Result<(), Self::Error>;
    fn encode_uint(&mut self, value: u32) -> Result<(), Self::Error>;
    fn encode_hyper(&mut self, value: i64) -> Result<(), Self::Error>;
    fn encode_uhyper(&mut self, value: u64) -> Result<(), Self::Error>;
    fn encode_f32(&mut self, value: f32) -> Result<(), Self::Error>;
    fn encode_f64(&mut self, value: f64) -> Result<(), Self::Error>;
    fn encode_bool(&mut self, value: bool) -> Result<(), Self::Error>;

    fn encode_opaque_fix(&mut self, value: &[u8]) -> Result<(), Self::Error>;
    fn encode_opaque_var(&mut self, value: &[u8]) -> Result<(), Self::Error>;

    fn encode_string(&mut self, value: &str) -> Result<(), Self::Error>;
}
