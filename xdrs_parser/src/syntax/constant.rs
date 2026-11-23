use std::ops::Deref;

use logos::Span;
use winnow::{
    ModalResult, Parser, combinator::alt, error::StrContext, token::any,
};

use crate::token::{SpannedToken, Token, TokenStream};

/// ```text
/// constant:
///    decimal-constant | hexadecimal-constant | octal-constant
/// ```
#[allow(non_snake_case)]
pub fn Const(input: &mut TokenStream<'_, '_>) -> ModalResult<Const> {
    alt((
        HexLit.map(Const::Hex),
        OctalLit.map(Const::Octal),
        DecimalLit.map(Const::Decimal),
    ))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Const {
    Hex(HexLit),
    Octal(OctalLit),
    Decimal(DecimalLit),
}

impl Const {
    pub fn value(&self) -> i64 {
        match self {
            Const::Hex(l) => l.value,
            Const::Octal(l) => l.value,
            Const::Decimal(l) => l.value,
        }
    }
}

#[allow(non_snake_case)]
pub fn HexLit(input: &mut TokenStream<'_, '_>) -> ModalResult<HexLit> {
    any.verify_map(|tok: SpannedToken<'_>| match tok.token() {
        Token::HexLit(lit) => Some((*lit, tok.span().clone())),
        _ => None,
    })
    .try_map(|(lit, span)| {
        let is_negative = lit.starts_with("-");
        let digits = lit
            .trim_start_matches("-")
            .trim_start_matches("0x")
            .trim_start_matches("0X");

        let value = i64::from_str_radix(digits, 16)?;
        let value = if is_negative { -value } else { value };
        Ok::<_, std::num::ParseIntError>(HexLit { value, span })
    })
    .context(StrContext::Label("hexadecimal integer literal"))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HexLit {
    pub value: i64,
    pub span: Span,
}

impl Deref for HexLit {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[allow(non_snake_case)]
pub fn OctalLit(input: &mut TokenStream<'_, '_>) -> ModalResult<OctalLit> {
    any.verify_map(|tok: SpannedToken<'_>| match tok.token() {
        Token::OctalLit(lit) => Some((*lit, tok.span().clone())),
        _ => None,
    })
    .try_map(|(lit, span)| {
        let is_negative = lit.starts_with("-");
        let digits =
            lit.trim_start_matches("-").strip_prefix("0").unwrap_or(lit);
        let value = i64::from_str_radix(digits, 8)?;
        let value = if is_negative { -value } else { value };
        Ok::<_, std::num::ParseIntError>(OctalLit { value, span })
    })
    .context(StrContext::Label("octal integer literal"))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OctalLit {
    pub value: i64,
    pub span: Span,
}

impl Deref for OctalLit {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[allow(non_snake_case)]
pub fn DecimalLit(input: &mut TokenStream<'_, '_>) -> ModalResult<DecimalLit> {
    any.verify_map(|tok: SpannedToken<'_>| match tok.token() {
        Token::DecimalLit(lit) => Some((*lit, tok.span().clone())),
        _ => None,
    })
    .try_map(|(lit, span)| {
        let is_negative = lit.starts_with("-");
        let digits = lit.trim_start_matches("-");
        let value = i64::from_str_radix(digits, 10)?;
        let value = if is_negative { -value } else { value };
        Ok::<_, std::num::ParseIntError>(DecimalLit { value, span })
    })
    .context(StrContext::Label("decimal integer literal"))
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DecimalLit {
    pub value: i64,
    pub span: Span,
}

impl Deref for DecimalLit {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type Const;
        pub fn constant_asserts(acc) {
            for (input, assert) in hex_lit_asserts() {
                asserts!(acc; type Const; { input.clone() }: Const::Hex(v) => assert(v));
            }
            for (input, assert) in octal_lit_asserts() {
                asserts!(acc; type Const; { input.clone() }: Const::Octal(v) => assert(v));
            }
            for (input, assert) in decimal_lit_asserts() {
                asserts!(acc; type Const; { input.clone() }: Const::Decimal(v) => assert(v));
            }
        }
    }

    define_asserts! {
        type OctalLit;
        pub fn octal_lit_asserts(acc) {
            asserts! {
                acc;
                type OctalLit;
                "-07462373": OctalLit { value: -0o7462373, .. },
                "-002": OctalLit { value: -0o02, .. },
                "-01": OctalLit { value: -0o1, .. },
                "00": OctalLit { value: 0o0, .. },
                "01": OctalLit { value: 0o1, .. },
                "002": OctalLit { value: 0o02, .. },
                "07462373": OctalLit { value: 0o7462373, .. },
            }
        }
    }

    define_asserts! {
        type HexLit;
        pub fn hex_lit_asserts(acc) {
            asserts! {
                acc;
                type HexLit;
                "-0xABCF": HexLit { value: -0xABCF, .. },
                "-0x02": HexLit { value: -0x02, .. },
                "-0x1": HexLit { value: -0x1, .. },
                "0x0": HexLit { value: 0x0, .. },
                "0x1": HexLit { value: 0x1, .. },
                "0x02": HexLit { value: 0x02, .. },
                "0xABCF": HexLit { value: 0xABCF, .. },
            }
        }
    }

    define_asserts! {
        type DecimalLit;
        pub fn decimal_lit_asserts(acc) {
            asserts! {
                acc;
                type DecimalLit;
                "-8462373": DecimalLit { value: -8462373, .. },
                "-2": DecimalLit { value: -2, .. },
                "-1": DecimalLit { value: -1, .. },
                "0": DecimalLit { value: 0, .. },
                "1": DecimalLit { value: 1, .. },
                "2": DecimalLit { value: 2, .. },
                "8462373": DecimalLit { value: 8462373, .. },
            }
        }
    }

    #[test]
    fn constant() {
        for (input, assert) in constant_asserts() {
            assert(&parse_str(Const, input).unwrap());
        }
    }

    #[test]
    fn hex_lit() {
        for (input, assert) in hex_lit_asserts() {
            assert(&parse_str(HexLit, input).unwrap());
        }

        let _ = parse_str(HexLit, "qwe").unwrap_err();
        let _ = parse_str(HexLit, "123").unwrap_err();
        let _ = parse_str(HexLit, "0123").unwrap_err();
    }

    #[test]
    fn octal_lit() {
        for (input, assert) in octal_lit_asserts() {
            assert(&parse_str(OctalLit, input).unwrap());
        }

        let _ = parse_str(OctalLit, "qwe").unwrap_err();
        let _ = parse_str(OctalLit, "123").unwrap_err();
        let _ = parse_str(OctalLit, "0x123").unwrap_err();
    }

    #[test]
    fn decimal_lit() {
        for (input, assert) in decimal_lit_asserts() {
            assert(&parse_str(DecimalLit, input).unwrap());
        }

        let _ = parse_str(DecimalLit, "qwe").unwrap_err();
        let _ = parse_str(DecimalLit, "0x123").unwrap_err();
        let _ = parse_str(DecimalLit, "0123").unwrap_err();
    }
}
