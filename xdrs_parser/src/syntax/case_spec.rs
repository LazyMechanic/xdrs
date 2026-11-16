use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// case-spec:
///   ( "case" value ":")
///   ( "case" value ":") *
///   declaration ";"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<CaseSpec> {
    todo!()
}

pub struct CaseSpec {}
