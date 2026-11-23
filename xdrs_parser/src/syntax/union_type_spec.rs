use winnow::{ModalResult, Parser, combinator::cut_err, seq};

use crate::{
    syntax::{token::Token, union_body::UnionBody},
    token::TokenStream,
};

/// ```text
/// union-type-spec:
///    "union" union-body
/// ```
#[allow(non_snake_case)]
pub fn UnionTypeSpec<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<UnionTypeSpec<'t>> {
    seq! {
        UnionTypeSpec {
            union: Token![union],
            body: cut_err(UnionBody),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnionTypeSpec<'t> {
    pub union: Token![union],
    pub body: UnionBody<'t>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        union_body::tests::union_body_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type UnionTypeSpec<'a>;
        pub fn union_type_spec_asserts(acc) {
            for (body_input, body_assert) in union_body_asserts() {
                asserts! { acc; type UnionTypeSpec<'a>; { format!("union {body_input}") }: UnionTypeSpec { body, .. } => body_assert(body) }
            }
        }
    }

    #[test]
    fn union_type_spec() {
        for (input, assert) in union_type_spec_asserts() {
            assert(&parse_str(UnionTypeSpec, input).unwrap());
        }

        let _ = parse_str(UnionTypeSpec, "switch (int v) { case 1: void; }");
    }
}
