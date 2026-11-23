use miette::SourceSpan;

use crate::token::TokenizeError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("failed to parse schema")]
pub enum ParseError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Tokenize(#[from] TokenizeError),

    #[error("syntax error")]
    Syntax(#[from] SyntaxError),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("syntax error")]
pub struct SyntaxError {
    #[label("{message}")]
    pub span: SourceSpan,
    pub message: String,
}
