mod attrs;
mod decode;
mod discriminant;
mod encode;
mod receivers;
mod utils;

use darling::{FromDeriveInput, util::SpannedValue};
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::{
    decode::impl_decode_derive, discriminant::impl_discriminant_derive,
    encode::impl_encode_derive, receivers::DeriveReceiver,
};

#[proc_macro_derive(Encode, attributes(xdrs))]
pub fn encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match SpannedValue::<DeriveReceiver>::from_derive_input(&input) {
        Ok(parsed) => match impl_encode_derive(parsed) {
            Ok(expanded) => expanded.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}

#[proc_macro_derive(Decode, attributes(xdrs))]
pub fn decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match SpannedValue::<DeriveReceiver>::from_derive_input(&input) {
        Ok(parsed) => match impl_decode_derive(parsed) {
            Ok(expanded) => expanded.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}

#[proc_macro_derive(Discriminant, attributes(xdrs))]
pub fn discriminant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match SpannedValue::<DeriveReceiver>::from_derive_input(&input) {
        Ok(parsed) => match impl_discriminant_derive(parsed) {
            Ok(expanded) => expanded.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}
