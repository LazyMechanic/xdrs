use xdrs::{Decode, DecodeError, Decoder};

#[repr(i32)]
enum Discriminant {
    Int,
    String,
    Void,
}

impl<'d> Decode<'d> for Discriminant {
    fn decode<D: Decoder<'d>>(decoder: &mut D) -> Result<Self, D::Error> {
        let val = i32::decode(decoder)?;
        match val {
            0 => Ok(Discriminant::Int),
            1 => Ok(Discriminant::String),
            2 => Ok(Discriminant::Void),
            unknown => Err(D::Error::unknown_variant(unknown)),
        }
    }
}

#[derive(Decode)]
#[xdrs(switch = "Discriminant")]
enum Derived {
    #[xdrs(discriminant = "Discriminant::Int")]
    Int(i32),
    #[xdrs(discriminant = "Discriminant::String")]
    String(String),
    #[xdrs(discriminant = "Discriminant::Void")]
    Void,
    #[xdrs(default)]
    Default,
}

fn main() {}
