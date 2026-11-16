use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// union-body:
///    "switch" "(" declaration ")" "{"
///       case-spec
///       case-spec *
///       [ "default" ":" declaration ";" ]
///    "}"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<UnionBody> {
    todo!()
}

pub struct UnionBody {}
