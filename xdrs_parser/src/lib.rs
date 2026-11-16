pub mod ast;
pub mod error;
pub mod token;

mod syntax;

use crate::{
    ast::Ast, error::ParseError, syntax::parse_tokens, token::tokenize,
};

pub fn parse_schema(input: &str) -> Result<Ast, ParseError> {
    let tokens = tokenize(input)?;
    let ast = parse_tokens(tokens)?;
    Ok(ast)
}
