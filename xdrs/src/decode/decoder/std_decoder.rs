use crate::{
    Error, ExpectedSize,
    decode::{
        Reference,
        decoder::{Decoder, Read},
        impls::check_size,
    },
    padding_len,
};

pub struct StdDecoder<R> {
    reader: R,
}

impl<R> StdDecoder<R> {
    pub fn new(reader: R) -> StdDecoder<R> {
        StdDecoder { reader }
    }
}

macro_rules! impl_num {
    ($reader:expr, $ty:ty) => {{
        const BYTES_COUNT: usize = std::mem::size_of::<$ty>();

        let slice = $reader.read_bytes(BYTES_COUNT)?;
        let slice = slice.as_ref();
        check_size::<Self::Error>(
            slice.len(),
            ExpectedSize::Exact(BYTES_COUNT),
        )?;

        let mut arr = [0u8; BYTES_COUNT];
        // SAFETY: read_bytes(BYTES_COUNT) returns exactly BYTES_COUNT bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                arr.as_mut_ptr(),
                BYTES_COUNT,
            )
        };

        Ok(<$ty>::from_be_bytes(arr))
    }};
}

impl<'d, R: Read<'d>> Decoder<'d> for StdDecoder<R> {
    type Error = Error;

    fn decode_int(&mut self) -> Result<i32, Self::Error> {
        impl_num!(self.reader, i32)
    }

    fn decode_uint(&mut self) -> Result<u32, Self::Error> {
        impl_num!(self.reader, u32)
    }

    fn decode_hyper(&mut self) -> Result<i64, Self::Error> {
        impl_num!(self.reader, i64)
    }

    fn decode_uhyper(&mut self) -> Result<u64, Self::Error> {
        impl_num!(self.reader, u64)
    }

    fn decode_f32(&mut self) -> Result<f32, Self::Error> {
        impl_num!(self.reader, f32)
    }

    fn decode_f64(&mut self) -> Result<f64, Self::Error> {
        impl_num!(self.reader, f64)
    }

    fn decode_bool(&mut self) -> Result<bool, Self::Error> {
        match self.decode_int()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(Self::Error::unknown_variant(other)),
        }
    }

    fn decode_opaque_fix<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'d, 's, [u8]>, Self::Error>
    where
        'd: 's,
    {
        check_size::<Self::Error>(len, ExpectedSize::Max(u32::MAX as usize))?;

        let padding_len = padding_len(len);
        if padding_len == 0 {
            return self.reader.read_bytes(len);
        }

        // max(sz) = u32::MAX
        // u32::MAX % 4 = 3 => max(sz + padding_len) = u32::MAX + 1

        let len_with_padding = len + padding_len;
        let data_with_padding = self.reader.read_bytes(len_with_padding)?;
        check_size::<Self::Error>(
            data_with_padding.as_ref().len(),
            ExpectedSize::Exact(len_with_padding),
        )?;

        // SAFETY: read_bytes(len_with_padding) returns exactly len_with_padding bytes.
        let data = match data_with_padding {
            Reference::Borrowed(d) => {
                Reference::Borrowed(unsafe { d.get_unchecked(..len) })
            }
            Reference::Copied(d) => {
                Reference::Copied(unsafe { d.get_unchecked(..len) })
            }
        };

        Ok(data)
    }

    fn decode_opaque_var<'s>(
        &'s mut self,
        maxlen: Option<usize>,
    ) -> Result<Reference<'d, 's, [u8]>, Self::Error>
    where
        'd: 's,
    {
        // Read size
        let len = self.decode_uint().map(|v| v as usize)?;
        if let Some(maxlen) = maxlen {
            check_size::<Self::Error>(len, ExpectedSize::Max(maxlen))?;
        }

        // Read data
        self.decode_opaque_fix(len)
    }

    fn decode_string<'s>(
        &'s mut self,
        maxlen: Option<usize>,
    ) -> Result<Reference<'d, 's, str>, Self::Error>
    where
        'd: 's,
    {
        let bytes = self.decode_opaque_var(maxlen)?;
        let str = match bytes {
            Reference::Borrowed(v) => Reference::Borrowed(
                str::from_utf8(v).map_err(|_| Error::bad_utf8())?,
            ),
            Reference::Copied(v) => Reference::Copied(
                str::from_utf8(v).map_err(|_| Error::bad_utf8())?,
            ),
        };
        Ok(str)
    }
}
