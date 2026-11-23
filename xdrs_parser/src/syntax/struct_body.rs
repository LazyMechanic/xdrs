use winnow::{
    ModalResult, Parser,
    combinator::{cut_err, repeat},
    error::StrContext,
    seq,
};

use crate::{
    syntax::{declaration::Declaration, token::Token},
    token::TokenStream,
};

/// ```text
/// struct-body:
///    "{"
///       ( declaration ";" )
///       ( declaration ";" )*
///    "}"
/// ```
#[allow(non_snake_case)]
pub fn StructBody<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<StructBody<'t>> {
    seq! {
        StructBody {
            lbrace: Token!['{'],
            fields: cut_err(repeat(1.., StructBodyField)),
            rbrace: cut_err(Token!['}']),
        }
    }
    .context(StrContext::Label("struct body"))
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn StructBodyField<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<StructBodyField<'t>> {
    seq! {
        StructBodyField {
            declaration: Declaration,
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructBody<'t> {
    pub lbrace: Token!['{'],
    pub fields: Vec<StructBodyField<'t>>,
    pub rbrace: Token!['}'],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructBodyField<'t> {
    pub declaration: Declaration<'t>,
    pub semi: Token![;],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        declaration::tests::declaration_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type StructBody<'a>;
        pub fn struct_body_asserts(acc) {
            let mut input = "{\n".to_string();
            for (field_input, _) in struct_body_field_asserts() {
                input = format!("{input}  {field_input}\n");
            }
            input.push('}');

            asserts! {
                acc;
                type StructBody<'a>;
                { input }:
                    StructBody { fields, .. }
                    => {
                        for (actual_field, (_, field_assert)) in fields.iter().zip(struct_body_field_asserts()) {
                            field_assert(actual_field);
                        }
                    },
            }
        }
    }

    define_asserts! {
        type StructBodyField<'a>;
        pub fn struct_body_field_asserts(acc) {
            for (decl_input, decl_assert) in declaration_asserts() {
                asserts! {
                    acc;
                    type StructBodyField<'a>;
                    { format!("{decl_input};") }:
                        StructBodyField { declaration, .. }
                        => decl_assert(&declaration),
                }
            }
        }
    }

    #[test]
    fn struct_body_field() {
        for (input, assert) in struct_body_field_asserts() {
            assert(&parse_str(StructBodyField, input).unwrap());
        }
    }

    #[test]
    fn struct_body() {
        for (input, assert) in struct_body_asserts() {
            assert(&parse_str(StructBody, input).unwrap());
        }
    }
}
