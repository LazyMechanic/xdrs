use std::fmt::{Display, Formatter, Write};

use logos::{Logos, Span};
use miette::SourceSpan;
use winnow::error::{StrContext, StrContextValue};

pub type TokenStream<'a, 't> = &'a [SpannedToken<'t>];

pub(crate) fn tokenize(
    input: &str,
) -> Result<Vec<SpannedToken<'_>>, TokenizeError> {
    let mut tokens = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut last_comment_end: Option<usize> = None;

    for (maybe_token, span) in RawToken::lexer(input).spanned() {
        let raw_token = maybe_token?;

        // If between last comment and current token was empty line then clean pending
        if let Some(prev_end) = last_comment_end {
            if input[prev_end..span.start].matches('\n').count() > 1 {
                pending_comments.clear();
            }
        }

        match raw_token {
            RawToken::LineComment(_) | RawToken::BlockComment(_) => {
                let lines = clean_comment(raw_token);

                pending_comments.extend(lines);
                last_comment_end = Some(span.end);
            }
            other => {
                let token = other.try_into().map_err(|_| {
                    TokenizeError::InvalidToken {
                        span: SourceSpan::from(span.clone()),
                    }
                })?;

                tokens.push(SpannedToken {
                    token,
                    span,
                    leading_comments: std::mem::take(&mut pending_comments),
                });

                last_comment_end = None;
            }
        }
    }

    Ok(tokens)
}

fn clean_comment(raw: RawToken<'_>) -> Vec<String> {
    match raw {
        RawToken::LineComment(s) => {
            let after_slashes = s.trim_start_matches('/');
            let text = after_slashes
                .strip_prefix(' ')
                .unwrap_or(after_slashes)
                .trim_end();
            vec![text.to_string()]
        }
        RawToken::BlockComment(s) => {
            let inner = s
                .trim_start_matches("/**")
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            inner
                .lines()
                .map(|line| {
                    let trimmed = line.trim();
                    let no_star = trimmed.strip_prefix('*').unwrap_or(trimmed);
                    no_star.strip_prefix(' ').unwrap_or(no_star).to_string()
                })
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

#[non_exhaustive]
#[derive(
    Default, Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic,
)]
#[error("failed to tokenize input")]
pub enum TokenizeError {
    #[error("processed invalid token")]
    InvalidToken {
        #[label("invalid syntax")]
        span: SourceSpan,
    },
    #[error("unknown error")]
    #[default]
    UnknownError,
}

impl TokenizeError {
    fn invalid_token<'a>(lex: &mut logos::Lexer<'a, RawToken<'a>>) -> Self {
        Self::InvalidToken {
            span: SourceSpan::from(lex.span()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Logos)]
#[logos(error(TokenizeError, callback = TokenizeError::invalid_token))]
#[logos(skip(r"[ \t\r\n\f]+"))] // skip whitespaces
enum RawToken<'a> {
    #[token("const")]
    Const,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("union")]
    Union,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("typedef")]
    Typedef,

    #[token("void")]
    Void,
    #[token("bool")]
    Bool,
    #[token("int")]
    Int,
    #[token("hyper")]
    Hyper,
    #[token("unsigned")]
    Unsigned,
    #[token("float")]
    Float,
    #[token("double")]
    Double,
    #[token("quadruple")]
    Quadruple,
    #[token("opaque")]
    Opaque,
    #[token("string")]
    String,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token("=")]
    Eq,
    #[token("*")]
    Star,
    #[token(",")]
    Comma,

    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment(&'a str),
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/")]
    BlockComment(&'a str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),

    #[regex(r"-?0x[0-9a-fA-F]+")]
    HexLit(&'a str),
    #[regex(r"-?0[0-7]+")]
    OctalLit(&'a str),
    #[regex(r"-?[0-9][0-9]*")]
    DecimalLit(&'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    Const,
    Struct,
    Enum,
    Union,
    Switch,
    Case,
    Default,
    Typedef,
    Void,
    Bool,
    Int,
    Hyper,
    Unsigned,
    Float,
    Double,
    Quadruple,
    Opaque,
    String,

    /// `(` Token
    LParen,
    /// `)` Token
    RParen,
    /// `{` Token
    LBrace,
    /// `]` Token
    RBrace,
    /// `[` Token
    LBracket,
    /// `]` Token
    RBracket,
    /// `<` Token
    LAngle,
    /// `>` Token
    RAngle,
    /// `;` Token
    Semi,
    /// `:` Token
    Colon,
    /// `=` Token
    Eq,
    /// `*` Token
    Star,
    /// `,` Token
    Comma,

    Ident(&'a str),

    /// `0xFF`
    HexLit(&'a str),
    /// `077`
    OctalLit(&'a str),
    /// `123`
    DecimalLit(&'a str),
}

impl Token<'_> {
    pub fn error_context(&self) -> StrContext {
        match self {
            Token::Const => StrContext::Expected("const".into()),
            Token::Struct => StrContext::Expected("struct".into()),
            Token::Enum => StrContext::Expected("enum".into()),
            Token::Union => StrContext::Expected("union".into()),
            Token::Switch => StrContext::Expected("switch".into()),
            Token::Case => StrContext::Expected("case".into()),
            Token::Default => StrContext::Expected("default".into()),
            Token::Typedef => StrContext::Expected("typedef".into()),
            Token::Void => StrContext::Expected("void".into()),
            Token::Bool => StrContext::Expected("bool".into()),
            Token::Int => StrContext::Expected("int".into()),
            Token::Hyper => StrContext::Expected("hyper".into()),
            Token::Unsigned => StrContext::Expected("unsigned".into()),
            Token::Float => StrContext::Expected("float".into()),
            Token::Double => StrContext::Expected("double".into()),
            Token::Quadruple => StrContext::Expected("quadruple".into()),
            Token::Opaque => StrContext::Expected("opaque".into()),
            Token::String => StrContext::Expected("string".into()),
            Token::LParen => StrContext::Expected("(".into()),
            Token::RParen => StrContext::Expected(")".into()),
            Token::LBrace => StrContext::Expected("{".into()),
            Token::RBrace => StrContext::Expected("}".into()),
            Token::LBracket => StrContext::Expected("[".into()),
            Token::RBracket => StrContext::Expected("]".into()),
            Token::LAngle => StrContext::Expected("<".into()),
            Token::RAngle => StrContext::Expected(">".into()),
            Token::Semi => StrContext::Expected(";".into()),
            Token::Colon => StrContext::Expected(":".into()),
            Token::Eq => StrContext::Expected("=".into()),
            Token::Star => StrContext::Expected("*".into()),
            Token::Comma => StrContext::Expected(",".into()),
            Token::Ident(_) => {
                StrContext::Expected(StrContextValue::Description("identifier"))
            }
            Token::HexLit(_) => StrContext::Expected(
                StrContextValue::Description("hex literal"),
            ),
            Token::OctalLit(_) => StrContext::Expected(
                StrContextValue::Description("octal literal"),
            ),
            Token::DecimalLit(_) => StrContext::Expected(
                StrContextValue::Description("decimal literal"),
            ),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Const => f.write_str("const"),
            Token::Struct => f.write_str("struct"),
            Token::Enum => f.write_str("enum"),
            Token::Union => f.write_str("union"),
            Token::Switch => f.write_str("switch"),
            Token::Case => f.write_str("case"),
            Token::Default => f.write_str("default"),
            Token::Typedef => f.write_str("typedef"),
            Token::Void => f.write_str("void"),
            Token::Bool => f.write_str("bool"),
            Token::Int => f.write_str("int"),
            Token::Hyper => f.write_str("hyper"),
            Token::Unsigned => f.write_str("unsigned"),
            Token::Float => f.write_str("float"),
            Token::Double => f.write_str("double"),
            Token::Quadruple => f.write_str("quadruple"),
            Token::Opaque => f.write_str("opaque"),
            Token::String => f.write_str("string"),
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::LBrace => f.write_str("{"),
            Token::RBrace => f.write_str("}"),
            Token::LBracket => f.write_str("["),
            Token::RBracket => f.write_str("]"),
            Token::LAngle => f.write_str("<"),
            Token::RAngle => f.write_str(">"),
            Token::Semi => f.write_str(";"),
            Token::Colon => f.write_str(":"),
            Token::Eq => f.write_str("="),
            Token::Star => f.write_str("*"),
            Token::Comma => f.write_str(","),
            Token::Ident(v) => f.write_str(v),
            Token::HexLit(v) => f.write_str(v),
            Token::OctalLit(v) => f.write_str(v),
            Token::DecimalLit(v) => f.write_str(v),
        }
    }
}

impl<'a> TryFrom<RawToken<'a>> for Token<'a> {
    type Error = RawToken<'a>;

    fn try_from(value: RawToken<'a>) -> Result<Self, Self::Error> {
        match value {
            RawToken::Const => Ok(Token::Const),
            RawToken::Struct => Ok(Token::Struct),
            RawToken::Enum => Ok(Token::Enum),
            RawToken::Union => Ok(Token::Union),
            RawToken::Switch => Ok(Token::Switch),
            RawToken::Case => Ok(Token::Case),
            RawToken::Default => Ok(Token::Default),
            RawToken::Typedef => Ok(Token::Typedef),
            RawToken::Void => Ok(Token::Void),
            RawToken::Bool => Ok(Token::Bool),
            RawToken::Int => Ok(Token::Int),
            RawToken::Hyper => Ok(Token::Hyper),
            RawToken::Unsigned => Ok(Token::Unsigned),
            RawToken::Float => Ok(Token::Float),
            RawToken::Double => Ok(Token::Double),
            RawToken::Quadruple => Ok(Token::Quadruple),
            RawToken::Opaque => Ok(Token::Opaque),
            RawToken::String => Ok(Token::String),
            RawToken::LParen => Ok(Token::LParen),
            RawToken::RParen => Ok(Token::RParen),
            RawToken::LBrace => Ok(Token::LBrace),
            RawToken::RBrace => Ok(Token::RBrace),
            RawToken::LBracket => Ok(Token::LBracket),
            RawToken::RBracket => Ok(Token::RBracket),
            RawToken::LAngle => Ok(Token::LAngle),
            RawToken::RAngle => Ok(Token::RAngle),
            RawToken::Semi => Ok(Token::Semi),
            RawToken::Colon => Ok(Token::Colon),
            RawToken::Eq => Ok(Token::Eq),
            RawToken::Star => Ok(Token::Star),
            RawToken::Comma => Ok(Token::Comma),
            RawToken::Ident(v) => Ok(Token::Ident(v)),
            RawToken::HexLit(v) => Ok(Token::HexLit(v)),
            RawToken::OctalLit(v) => Ok(Token::OctalLit(v)),
            RawToken::DecimalLit(v) => Ok(Token::DecimalLit(v)),
            unexpected => Err(unexpected),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpannedToken<'a> {
    token: Token<'a>,
    span: Span,
    leading_comments: Vec<String>,
}

impl<'a> SpannedToken<'a> {
    pub fn token<'s>(&'s self) -> &'s Token<'a> {
        &self.token
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
