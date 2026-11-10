mod bounded;
pub mod decode;
pub mod encode;
mod error;

#[cfg(any(test, feature = "__test_utils"))]
pub mod test_utils;

use std::fmt::{Display, Formatter};

pub use xdrs_derive::{Decode, Encode};

pub use crate::{
    bounded::Bounded,
    decode::{
        Decode, DecodeError, DecodeOwned, Decoder, StdDecoder, from_reader,
        from_slice,
    },
    encode::{Encode, EncodeError, Encoder, StdEncoder, to_vec, to_writer},
    error::Error,
};

static PADDING_MAX_LEN: usize = 4;
fn padding(len: usize) -> &'static [u8] {
    static PADDING: [u8; PADDING_MAX_LEN] = [0; PADDING_MAX_LEN];
    &PADDING[..padding_len(len)]
}
fn padding_len(len: usize) -> usize {
    (PADDING_MAX_LEN - (len % PADDING_MAX_LEN)) % PADDING_MAX_LEN
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExpectedSize {
    Exact(usize),
    Max(usize),
}

impl ExpectedSize {
    pub(crate) fn display(self) -> impl Display {
        struct DisplayImpl(ExpectedSize);
        impl Display for DisplayImpl {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self.0 {
                    ExpectedSize::Exact(expsz) => {
                        write!(f, "actual size must be exactly {expsz}")
                    }
                    ExpectedSize::Max(expsz) => {
                        write!(f, "actual size must be at most {expsz}")
                    }
                }
            }
        }

        DisplayImpl(self)
    }
}

#[doc(hidden)]
pub mod macro_reexport {
    pub use core::{option::Option, result::Result, stringify};
    pub use std::io::Write;
}

mod private {
    pub trait Sealed {}
}
