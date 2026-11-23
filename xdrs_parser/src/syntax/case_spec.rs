use winnow::{
    ModalResult, Parser,
    combinator::{cut_err, repeat},
    seq,
};

use crate::{
    syntax::{declaration::Declaration, token::Token, value::Value},
    token::TokenStream,
};

/// ```text
/// case-spec:
///   ( "case" value ":")
///   ( "case" value ":") *
///   declaration ";"
/// ```
#[allow(non_snake_case)]
pub fn CaseSpec<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<CaseSpec<'t>> {
    seq! {
        CaseSpec {
            cases: repeat(1.., Case),
            declaration: cut_err(Declaration),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Case<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Case<'t>> {
    seq! {
        Case {
            case: Token![case],
            value: cut_err(Value),
            colon: cut_err(Token![:]),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CaseSpec<'t> {
    pub cases: Vec<Case<'t>>,
    pub declaration: Declaration<'t>,
    pub semi: Token![;],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Case<'t> {
    pub case: Token![case],
    pub value: Value<'t>,
    pub colon: Token![:],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type CaseSpec<'a>;
        pub fn case_spec_asserts(acc) {
            asserts! {
                acc;
                type CaseSpec<'a>;
                "case 123: void;": CaseSpec { cases, declaration: Declaration::Void(_), .. }
                    if matches!(
                        cases.as_slice(),
                        [Case { value: Value::Const(_), .. }]
                    ),
                r#"
                    case 123:
                    case CONSTANT:
                        void;
                "#: CaseSpec { cases, declaration: Declaration::Void(_), .. }
                    if matches!(
                        cases.as_slice(),
                        [
                            Case { value: Value::Const(_), .. },
                            Case { value: Value::Ident(_), .. }
                        ]
                    ),
                r#"
                    case 1:
                    case 2:
                    case 3:
                    case 4:
                        void;
                "#: CaseSpec { cases, declaration: Declaration::Void(_), .. }
                    if matches!(
                        cases.as_slice(),
                        [
                            Case { value: Value::Const(_), .. },
                            Case { value: Value::Const(_), .. },
                            Case { value: Value::Const(_), .. },
                            Case { value: Value::Const(_), .. },
                        ]
                    ),
            }
        }
    }
    define_asserts! {
        type Case<'a>;
        pub fn case_asserts(acc) {
            asserts! {
                acc;
                type Case<'a>;
                "case 123:": Case { value: Value::Const(_), .. },
                "case CONST:": Case { value: Value::Ident(_), .. },
            }
        }
    }

    #[test]
    fn case() {
        for (input, assert) in case_asserts() {
            assert(&parse_str(Case, input).unwrap());
        }

        let _ = parse_str(Case, "CONSTANT:").unwrap_err();
        let _ = parse_str(Case, "case :").unwrap_err();
        let _ = parse_str(Case, "case CONSTANT").unwrap_err();
    }

    #[test]
    fn case_spec() {
        for (input, assert) in case_spec_asserts() {
            assert(&parse_str(CaseSpec, input).unwrap());
        }

        let _ = parse_str(CaseSpec, "void;").unwrap_err();
        let _ = parse_str(CaseSpec, "case 123:").unwrap_err();
        let _ = parse_str(CaseSpec, "case 123:").unwrap_err();
    }
}
