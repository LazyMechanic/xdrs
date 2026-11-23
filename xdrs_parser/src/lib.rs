pub mod error;
pub mod syntax;
pub mod token;

use crate::{
    error::ParseError,
    syntax::{Ast, parse_tokens},
    token::tokenize,
};

pub fn parse_schema(input: &str) -> Result<Ast<'_>, ParseError> {
    let tokens = tokenize(input)?;
    let ast = parse_tokens(&mut tokens.as_slice())?;
    Ok(ast)
}
