use winnow::{ModalResult, Parser, combinator::cut_err, seq};

use crate::{
    syntax::{
        ident::Ident,
        token::Token,
        utils::{Punctuated, separated1},
        value::Value,
    },
    token::TokenStream,
};

/// ```text
/// enum-body:
///    "{"
///       ( identifier "=" value )
///       ( "," identifier "=" value )*
///    "}"
/// ```
#[allow(non_snake_case)]
pub fn EnumBody<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<EnumBody<'t>> {
    seq! {
        EnumBody {
            lbrace: Token!['{'],
            variants: cut_err(separated1(EnumVariant, Token![,])),
            rbrace: cut_err(Token!['}']),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn EnumVariant<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<EnumVariant<'t>> {
    seq! {
        EnumVariant {
            ident: Ident,
            eq: Token![=],
            value: Value,
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumBody<'t> {
    pub lbrace: Token!['{'],
    pub variants: Punctuated<EnumVariant<'t>, Token![,]>,
    pub rbrace: Token!['}'],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumVariant<'t> {
    pub ident: Ident<'t>,
    pub eq: Token![=],
    pub value: Value<'t>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        utils::{asserts, define_asserts, parse_str},
        value::tests::value_asserts,
    };

    define_asserts! {
        type EnumBody<'a>;
        pub fn enum_body_asserts(acc) {
            let mut input = "{".to_string();
            for (idx, (variant_input, _)) in enum_variant_asserts().iter().enumerate() {
                if idx == 0 {
                    input = format!("{input} {variant_input}\n")
                } else {
                    input = format!("{input} , {variant_input}\n")
                }
            }
            input.push('}');

            asserts! {
                acc;
                type EnumBody<'a>;
                { input }:
                    EnumBody { variants, .. }
                    => {
                        for (actual_variant, (_, variant_assert)) in variants.iter().zip(enum_variant_asserts()) {
                            variant_assert(actual_variant);
                        }
                    },
            }
        }
    }

    define_asserts! {
        type EnumVariant<'a>;
        pub fn enum_variant_asserts(acc) {
            for (value_input, value_assert) in value_asserts() {
                asserts! {
                    acc;
                    type EnumVariant<'a>;
                    { format!("ident = {value_input}") }:
                        EnumVariant { ident, value, .. }
                        if &**ident == "ident"
                        => value_assert(&value),
                }
            }
        }
    }

    #[test]
    fn enum_variant() {
        for (input, assert) in enum_variant_asserts() {
            assert(&parse_str(EnumVariant, input).unwrap());
        }

        let _ = parse_str(EnumBody, "= 1").unwrap_err();
        let _ = parse_str(EnumBody, "VARIANT 1").unwrap_err();
        let _ = parse_str(EnumBody, "VARIANT =").unwrap_err();
    }

    #[test]
    fn enum_body() {
        for (input, assert) in enum_body_asserts() {
            assert(&parse_str(EnumBody, input).unwrap());
        }

        let _ = parse_str(EnumBody, " VARIANT = 1 }").unwrap_err();
        let _ = parse_str(EnumBody, "{  }").unwrap_err();
        let _ = parse_str(EnumBody, "{ VARIANT = 1").unwrap_err();
    }
}
