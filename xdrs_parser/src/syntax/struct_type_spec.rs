use winnow::{
    ModalResult, Parser, combinator::cut_err, error::StrContext, seq,
};

use crate::{
    syntax::{struct_body::StructBody, token::Token},
    token::TokenStream,
};

/// ```text
/// struct-type-spec:
///    "struct" struct-body
/// ```
#[allow(non_snake_case)]
pub fn StructTypeSpec<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<StructTypeSpec<'t>> {
    seq! {
        StructTypeSpec {
            struct_: Token![struct],
            body: cut_err(StructBody),
        }
    }
    .context(StrContext::Label("struct type specification"))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructTypeSpec<'a> {
    pub struct_: Token![struct],
    pub body: StructBody<'a>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        struct_body::tests::struct_body_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type StructTypeSpec<'a>;
        pub fn struct_type_spec_asserts(acc) {
            for (body_input, body_assert) in struct_body_asserts() {
                asserts! { acc; type StructTypeSpec<'a>; { format!("struct {body_input}") }: StructTypeSpec { body, .. } => body_assert(body) }
            }
        }
    }

    #[test]
    fn struct_type_spec() {
        for (input, assert) in struct_type_spec_asserts() {
            assert(&parse_str(StructTypeSpec, input).unwrap());
        }

        let _ = parse_str(StructTypeSpec, "{ int field; }").unwrap_err();
    }
}
