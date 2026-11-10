mod io_reader;
mod slice_reader;

pub use self::{io_reader::IoReader, slice_reader::SliceReader};
use crate::{Error, decode::Reference, private::Sealed};

pub trait Read<'r>: Sealed {
    #[doc(hidden)]
    fn read_bytes<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'r, 's, [u8]>, Error>;
}
