use winnow::{
    ModalResult, Parser,
    combinator::{cut_err, opt, repeat},
    seq,
};

use crate::{
    syntax::{case_spec::CaseSpec, declaration::Declaration, token::Token},
    token::TokenStream,
};

/// ```text
/// union-body:
///    "switch" "(" declaration ")" "{"
///       case-spec
///       case-spec *
///       [ "default" ":" declaration ";" ]
///    "}"
/// ```
#[allow(non_snake_case)]
pub fn UnionBody<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<UnionBody<'t>> {
    seq! {
        UnionBody {
            switch: Token![switch],
            lparen: cut_err(Token!['(']),
            declaration: cut_err(Declaration),
            rparen: cut_err(Token![')']),
            lbrace: cut_err(Token!['{']),
            cases: cut_err(repeat(1.., CaseSpec)),
            default: opt(DefaultBranch),
            rbrace: cut_err(Token!['}']),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DefaultBranch<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DefaultBranch<'t>> {
    seq! {
        DefaultBranch {
            default: Token![default],
            colon: cut_err(Token![:]),
            declaration: cut_err(Declaration),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnionBody<'t> {
    pub switch: Token![switch],
    pub lparen: Token!['('],
    pub declaration: Declaration<'t>,
    pub rparen: Token![')'],
    pub lbrace: Token!['{'],
    pub cases: Vec<CaseSpec<'t>>,
    pub default: Option<DefaultBranch<'t>>,
    pub rbrace: Token!['}'],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DefaultBranch<'t> {
    pub default: Token![default],
    pub colon: Token![:],
    pub declaration: Declaration<'t>,
    pub semi: Token![;],
}

#[cfg(test)]
pub mod tests {
    use std::borrow::Borrow;

    use super::*;
    use crate::syntax::{
        case_spec::tests::case_spec_asserts,
        declaration::tests::declaration_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type UnionBody<'a>;
        pub fn union_body_asserts(acc) {
            let case_inputs = case_spec_asserts()
                .keys()
                .map(|input| input.borrow())
                .collect::<Vec<_>>()
                .join("\n");
            for (decl_input, decl_assert) in declaration_asserts() {
                for (default_input, default_assert) in default_branch_asserts() {
                    asserts! {
                        acc;
                        type UnionBody<'a>;
                        { format!(r#"
                            switch ({decl_input}) {{
                                {case_inputs}
                                {default_input}
                            }}
                        "#) }:
                            UnionBody { declaration, cases, default: Some(default), .. }
                            => {
                                decl_assert(declaration);
                                default_assert(default);
                                for (actual_value, assert) in cases.iter().zip(case_spec_asserts().values()) {
                                    assert(actual_value)
                                }
                            },
                    }
                }
            }
        }
    }

    define_asserts! {
        type DefaultBranch<'a>;
        pub fn default_branch_asserts(acc) {
            for (decl_input, decl_assert) in declaration_asserts() {
                asserts! {
                    acc;
                    type DefaultBranch<'a>;
                    { format!("default: {decl_input};") }:
                        DefaultBranch { declaration, .. }
                        => decl_assert(declaration),
                }
            }
        }
    }

    #[test]
    fn default_branch() {
        for (input, assert) in default_branch_asserts() {
            assert(&parse_str(DefaultBranch, input).unwrap());
        }
    }

    #[test]
    fn union_body() {
        for (input, assert) in union_body_asserts() {
            assert(&parse_str(UnionBody, input).unwrap());
        }
    }
}
