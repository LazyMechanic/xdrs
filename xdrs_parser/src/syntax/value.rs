use winnow::{
    ModalResult, Parser,
    combinator::alt,
    error::{StrContext, StrContextValue},
};

use crate::{
    syntax::{constant::Const, ident::Ident},
    token::TokenStream,
};

/// ```text
/// value:
///      constant
///    | identifier
/// ```
#[allow(non_snake_case)]
pub fn Value<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Value<'t>> {
    alt((Const.map(Value::Const), Ident.map(Value::Ident)))
        .context(StrContext::Expected(StrContextValue::Description(
            "hex/octal/decimal value or constant identifier",
        )))
        .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value<'a> {
    Const(Const),
    Ident(Ident<'a>),
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        constant::tests::constant_asserts,
        ident::tests::ident_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type Value<'a>;
        pub fn value_asserts(acc) {
            for (input, assert) in constant_asserts() {
                asserts!(acc; type Value<'a>; { input.clone() }: Value::Const(v) => assert(v));
            }
            for (input, assert) in ident_asserts() {
                asserts!(acc; type Value<'a>; { input.clone() }: Value::Ident(v) => assert(v));
            }
        }
    }

    #[test]
    fn value() {
        for (input, assert) in value_asserts() {
            assert(&parse_str(Value, input).unwrap());
        }
    }
}
