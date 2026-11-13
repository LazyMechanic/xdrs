use std::fmt::Debug;

use crate::{
    Decode, Decoder, Encode, Error, ExpectedSize, StdEncoder,
    decode::{DecodeDiscriminant, Reference},
    padding_len,
};

#[track_caller]
pub fn assert_encode_eq<A: Encode, E: Encode>(actual: A, expected: E) {
    let mut act_encoder = StdEncoder::new(Vec::new());
    let mut exp_encoder = StdEncoder::new(Vec::new());

    actual.encode(&mut act_encoder).unwrap();
    expected.encode(&mut exp_encoder).unwrap();
    assert_eq!(
        act_encoder.into_inner(),
        exp_encoder.into_inner(),
        "encoded value doesn't match"
    );
}

#[track_caller]
pub fn assert_encode_err<A: Encode>(value: A) {
    let mut encoder = StdEncoder::new(Vec::new());
    let _ = value
        .encode(&mut encoder)
        .expect_err("value should encode with error");
}

#[track_caller]
pub fn assert_encode<T: Encode>(actual: T, expected: &[u8]) {
    let mut writer = StdEncoder::new(Vec::new());
    T::encode(&actual, &mut writer).unwrap();

    assert_eq!(writer.into_inner(), expected, "encoded value doesn't match");
}

#[track_caller]
pub fn assert_decode<'d, T: Decode<'d> + PartialEq + Debug>(
    input: &'d [u8],
    expected: T,
) {
    let mut decoder = TestSliceDecoder::new(input);

    let actual = T::decode(&mut decoder).unwrap();
    assert_eq!(actual, expected, "decoded value doesn't match");

    let expected_read_len = decoder.len();
    let actual_read_len = decoder.read_len();
    assert_eq!(
        actual_read_len, expected_read_len,
        "read length doesn't match"
    );
}

#[track_caller]
pub fn assert_decode_eq<'d, R, T>(reference_encoded: R, expected: T)
where
    T: for<'a> Decode<'a> + PartialEq + Debug,
    R: Encode,
{
    let mut encoder = StdEncoder::new(Vec::new());
    reference_encoded.encode(&mut encoder).unwrap();

    let encoded = encoder.into_inner();
    let mut decoder = TestSliceDecoder::new(&encoded);

    let actual = T::decode(&mut decoder).unwrap();
    assert_eq!(actual, expected, "decoded value doesn't match");

    let expected_read_len = decoder.len();
    let actual_read_len = decoder.read_len();
    assert_eq!(
        actual_read_len, expected_read_len,
        "read length doesn't match"
    );
}

#[track_caller]
pub fn assert_decode_discriminant<
    'd,
    T: DecodeDiscriminant<'d> + PartialEq + Debug,
>(
    input: &'d [u8],
    expected: (Option<T>, i32),
) {
    let mut decoder = TestSliceDecoder::new(input);

    let actual = T::decode_discriminant(&mut decoder).unwrap();
    assert_eq!(actual, expected, "decoded value doesn't match");

    let expected_read_len = decoder.len();
    let actual_read_len = decoder.read_len();
    assert_eq!(
        actual_read_len, expected_read_len,
        "read length doesn't match"
    );
}

///////

pub struct TestSliceDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> TestSliceDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn read_len(&self) -> usize {
        self.pos
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let begin = self.pos;
        let end = self
            .pos
            .checked_add(len)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| {
                Error::custom(
                    format!(
                        "the source buffer has fewer bytes remaining than \
                         requested; remaining: {remaining}, requested: {requested}",
                        remaining = self.data.len() - begin,
                        requested = len,
                    ),
                )
            })?;

        let buf = self.data.get(begin..end).expect("no data");

        self.pos = end;

        Ok(buf)
    }
}

macro_rules! impl_num {
    ($self:expr, $ty:ty) => {{
        const BYTES_COUNT: usize = std::mem::size_of::<$ty>();

        let slice = $self.read_bytes(BYTES_COUNT)?;
        let arr: [u8; BYTES_COUNT] = slice
            .try_into()
            .expect(&format!("slice must be of length {BYTES_COUNT}"));

        Ok(<$ty>::from_be_bytes(arr))
    }};
}

impl<'d> Decoder<'d> for TestSliceDecoder<'d> {
    type Error = Error;

    fn decode_int(&mut self) -> Result<i32, Self::Error> {
        impl_num!(self, i32)
    }

    fn decode_uint(&mut self) -> Result<u32, Self::Error> {
        impl_num!(self, u32)
    }

    fn decode_hyper(&mut self) -> Result<i64, Self::Error> {
        impl_num!(self, i64)
    }

    fn decode_uhyper(&mut self) -> Result<u64, Self::Error> {
        impl_num!(self, u64)
    }

    fn decode_f32(&mut self) -> Result<f32, Self::Error> {
        impl_num!(self, f32)
    }

    fn decode_f64(&mut self) -> Result<f64, Self::Error> {
        impl_num!(self, f64)
    }

    fn decode_bool(&mut self) -> Result<bool, Self::Error> {
        match self.decode_int()? {
            0 => Ok(false),
            1 => Ok(true),
            other => {
                Err(Error::custom(format_args!("unknown variant: {other}")))
            }
        }
    }

    fn decode_opaque_fix<'s>(
        &'s mut self,
        len: usize,
    ) -> Result<Reference<'d, 's, [u8]>, Self::Error>
    where
        'd: 's,
    {
        let padding_len = padding_len(len);
        if padding_len == 0 {
            return Ok(Reference::Borrowed(self.read_bytes(len)?));
        }

        let len_with_padding = len + padding_len;
        let data_with_padding = self.read_bytes(len_with_padding)?;
        Ok(Reference::Borrowed(data_with_padding.get(..len).expect(
            &format!("slice must have a length of at least {len}"),
        )))
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
        if let Some(maxlen) = maxlen
            && len > maxlen
        {
            return Err(Error::invalid_size(len, ExpectedSize::Max(maxlen)));
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
            Reference::Borrowed(v) => {
                Reference::Borrowed(str::from_utf8(v).map_err(|err| {
                    Error::custom(format_args!("bad utf8: {err}"))
                })?)
            }
            Reference::Copied(v) => {
                Reference::Copied(str::from_utf8(v).map_err(|err| {
                    Error::custom(format_args!("bad utf8: {err}"))
                })?)
            }
        };
        Ok(str)
    }
}
