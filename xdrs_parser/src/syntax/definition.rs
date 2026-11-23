use winnow::{ModalResult, Parser, combinator::alt};

use crate::{
    syntax::{const_def::ConstDef, type_def::TypeDef},
    token::TokenStream,
};

/// ```text
/// definition:
///      type-def
///    | constant-def
/// ```
#[allow(non_snake_case)]
pub fn Definition<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Definition<'t>> {
    alt((
        TypeDef.map(Definition::TypeDef),
        ConstDef.map(Definition::ConstDef),
    ))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Definition<'t> {
    TypeDef(TypeDef<'t>),
    ConstDef(ConstDef<'t>),
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        const_def::tests::const_def_asserts,
        type_def::tests::type_def_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type Definition<'a>;
        pub fn definition_asserts(acc) {
            for (input, assert) in type_def_asserts() {
                asserts! { acc; type Definition<'a>; { input.clone() }: Definition::TypeDef(v) => assert(&v) }
            }
            for (input, assert) in const_def_asserts() {
                asserts! { acc; type Definition<'a>; { input.clone() }: Definition::ConstDef(v) => assert(&v) }
            }
        }
    }

    #[test]
    fn definition() {
        for (input, assert) in definition_asserts() {
            assert(&parse_str(Definition, input).unwrap());
        }
    }
}
