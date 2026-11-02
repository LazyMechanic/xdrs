mod slice_read;

pub use self::slice_read::SliceRead;
use crate::{Error, ExpectedVariant};

macro_rules! impl_num {
    ($self:expr, $ty:ty) => {{
        const BYTES_COUNT: usize = std::mem::size_of::<$ty>();

        let (slice, n) = $self.read_bytes(BYTES_COUNT)?;
        debug_assert!(n == BYTES_COUNT, "not expected padding");

        let mut arr = [0u8; BYTES_COUNT];
        // SAFETY: read_fix(BYTES_COUNT) returns exactly BYTES_COUNT bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                arr.as_mut_ptr(),
                BYTES_COUNT,
            )
        };

        Ok((<$ty>::from_be_bytes(arr), BYTES_COUNT))
    }};
}

pub trait Read<'a> {
    fn read_bytes(&mut self, len: usize) -> Result<(&'a [u8], usize), Error>;

    fn read_i32(&mut self) -> Result<(i32, usize), Error> {
        impl_num!(self, i32)
    }

    fn read_u32(&mut self) -> Result<(u32, usize), Error> {
        impl_num!(self, u32)
    }

    fn read_i64(&mut self) -> Result<(i64, usize), Error> {
        impl_num!(self, i64)
    }

    fn read_u64(&mut self) -> Result<(u64, usize), Error> {
        impl_num!(self, u64)
    }

    fn read_f32(&mut self) -> Result<(f32, usize), Error> {
        impl_num!(self, f32)
    }

    fn read_f64(&mut self) -> Result<(f64, usize), Error> {
        impl_num!(self, f64)
    }

    fn read_bool(&mut self) -> Result<(bool, usize), Error> {
        let (v, n) = self.read_i32()?;
        match v {
            0 => Ok((false, n)),
            1 => Ok((true, n)),
            v => Err(Error::unknown_variant(
                v,
                &[ExpectedVariant::Value(0), ExpectedVariant::Value(1)],
            )),
        }
    }
}
