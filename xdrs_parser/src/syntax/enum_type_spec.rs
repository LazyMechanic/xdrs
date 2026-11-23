use winnow::{ModalResult, Parser, combinator::cut_err, seq};

use crate::{
    syntax::{enum_body::EnumBody, token::Token},
    token::TokenStream,
};

/// ```text
/// enum-type-spec:
///    "enum" enum-body
/// ```
#[allow(non_snake_case)]
pub fn EnumTypeSpec<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<EnumTypeSpec<'t>> {
    seq! {
        EnumTypeSpec {
            enum_: Token![enum],
            body: cut_err(EnumBody),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumTypeSpec<'t> {
    pub enum_: Token![enum],
    pub body: EnumBody<'t>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        enum_body::tests::enum_body_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type EnumTypeSpec<'a>;
        pub fn enum_type_spec_asserts(acc) {
            for (body_input, body_assert) in enum_body_asserts() {
                asserts! { acc; type EnumTypeSpec<'a>; { format!("enum {body_input}") }: EnumTypeSpec { body, .. } => body_assert(body) }
            }
        }
    }

    #[test]
    fn enum_type_spec() {
        for (input, assert) in enum_type_spec_asserts() {
            assert(&parse_str(EnumTypeSpec, input).unwrap());
        }
    }
}
