use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// definition:
///      type-def
///    | constant-def
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<Definition> {
    todo!()
}

pub struct Definition {}
