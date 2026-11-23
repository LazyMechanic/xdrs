use winnow::{ModalResult, Parser, combinator::cut_err, seq};

use crate::{
    syntax::{constant::Const, ident::Ident, token::Token},
    token::TokenStream,
};

/// ```text
/// constant-def:
///    "const" identifier "=" constant ";"
/// ```
#[allow(non_snake_case)]
pub fn ConstDef<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<ConstDef<'t>> {
    seq! {
        ConstDef {
            const_: Token![const],
            ident: cut_err(Ident),
            eq: cut_err(Token![=]),
            value: cut_err(Const),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstDef<'t> {
    pub const_: Token![const],
    pub ident: Ident<'t>,
    pub eq: Token![=],
    pub value: Const,
    pub semi: Token![;],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type ConstDef<'a>;
        pub fn const_def_asserts(acc) {
            asserts! {
                acc;
                type ConstDef<'a>;
                "const CONST = 123;": ConstDef { ident, value, .. } if &**ident == "CONST" && value.value() == 123,
            }
        }
    }

    #[test]
    fn const_def() {
        for (input, assert) in const_def_asserts() {
            assert(&parse_str(ConstDef, input).unwrap());
        }

        let _ = parse_str(ConstDef, "CONST = 123;").unwrap_err();
        let _ = parse_str(ConstDef, "const = 123;").unwrap_err();
        let _ = parse_str(ConstDef, "const CONST 123;").unwrap_err();
        let _ = parse_str(ConstDef, "const CONST = ;").unwrap_err();
        let _ = parse_str(ConstDef, "const CONST = 123").unwrap_err();
    }
}
