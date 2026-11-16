use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// constant-def:
///    "const" identifier "=" constant ";"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<ConstDef> {
    todo!()
}

pub struct ConstDef {}
