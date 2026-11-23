use winnow::{ModalResult, Parser, combinator::repeat, seq};

use crate::{syntax::definition::Definition, token::TokenStream};

/// ```text
/// specification:
///      definition *
/// ```
#[allow(non_snake_case)]
pub fn Specification<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Specification<'t>> {
    seq! {
        Specification {
            definitions: repeat(0.., Definition),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Specification<'t> {
    pub definitions: Vec<Definition<'t>>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        definition::tests::definition_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type Specification<'a>;
        pub fn specification_asserts(acc) {
            let mut input = String::from("");
            for def_input in definition_asserts().keys() {
                input = format!("{input}\n{def_input}")
            }

            asserts! {
                acc;
                type Specification<'a>;
                { input }:
                    Specification { definitions }
                    => {
                        for (actual_def, def_assert) in definitions.iter().zip(definition_asserts().values()) {
                            def_assert(actual_def)
                        }
                    }
            }
        }
    }

    #[test]
    fn specification() {
        for (input, assert) in specification_asserts() {
            assert(&parse_str(Specification, input).unwrap());
        }
    }
}
