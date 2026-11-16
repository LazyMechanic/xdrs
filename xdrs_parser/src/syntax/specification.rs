use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// specification:
///      definition *
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<Specification> {
    todo!()
}

pub struct Specification {}
