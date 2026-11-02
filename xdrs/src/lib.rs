pub mod decode;
pub mod encode;
pub mod error;
pub mod limited;
pub mod void;

pub use xdrs_derive::Encode;

pub use crate::{
    decode::{Decode, read::Read},
    encode::Encode,
    error::{Error, ExpectedSize, ExpectedVariant, ResultExt},
    limited::Limited,
    void::Void,
};

static PADDING_MAX_LEN: usize = 4;
fn padding(len: usize) -> &'static [u8] {
    static PADDING: [u8; PADDING_MAX_LEN] = [0; PADDING_MAX_LEN];
    &PADDING[..(PADDING_MAX_LEN - (len % PADDING_MAX_LEN)) % PADDING_MAX_LEN]
}

#[doc(hidden)]
pub mod reexport {
    pub use core::{option::Option, result::Result, stringify};
    pub use std::io::{Read, Write};
}

mod private {
    pub trait Sealed {}

    macro_rules! impl_sealed {
        ($($ty:ty),+$(,)?) => {
            $(impl $crate::private::Sealed for $ty {})+
        }
    }
    pub(crate) use impl_sealed;
}
