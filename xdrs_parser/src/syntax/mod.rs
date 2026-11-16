use crate::{ast::Ast, error::ParseError, token::SpannedToken};

pub(crate) fn parse_tokens(
    tokens: Vec<SpannedToken<'_>>,
) -> Result<Ast, ParseError> {
    todo!()
}
