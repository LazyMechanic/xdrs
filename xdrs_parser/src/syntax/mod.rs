//! Syntax
//!
//! ```text
//! declaration:
//!      type-specifier identifier
//!    | type-specifier identifier "[" value "]"
//!    | type-specifier identifier "<" [ value ] ">"
//!    | "opaque" identifier "[" value "]"
//!    | "opaque" identifier "<" [ value ] ">"
//!    | "string" identifier "<" [ value ] ">"
//!    | type-specifier "*" identifier
//!    | "void"
//!
//! value:
//!      constant
//!    | identifier
//!
//! constant:
//!    decimal-constant | hexadecimal-constant | octal-constant
//!
//! type-specifier:
//!      [ "unsigned" ] "int"
//!    | [ "unsigned" ] "hyper"
//!    | "float"
//!    | "double"
//!    | "quadruple"
//!    | "bool"
//!    | enum-type-spec
//!    | struct-type-spec
//!    | union-type-spec
//!    | identifier
//!
//! enum-type-spec:
//!    "enum" enum-body
//!
//! enum-body:
//!    "{"
//!       ( identifier "=" value )
//!       ( "," identifier "=" value )*
//!    "}"
//!
//! struct-type-spec:
//!    "struct" struct-body
//!
//! struct-body:
//!    "{"
//!       ( declaration ";" )
//!       ( declaration ";" )*
//!    "}"
//!
//! union-type-spec:
//!    "union" union-body
//!
//! union-body:
//!    "switch" "(" declaration ")" "{"
//!       case-spec
//!       case-spec *
//!       [ "default" ":" declaration ";" ]
//!    "}"
//!
//! case-spec:
//!   ( "case" value ":")
//!   ( "case" value ":") *
//!   declaration ";"
//!
//! constant-def:
//!    "const" identifier "=" constant ";"
//!
//! type-def:
//!      "typedef" declaration ";"
//!    | "enum" identifier enum-body ";"
//!    | "struct" identifier struct-body ";"
//!    | "union" identifier union-body ";"
//!
//! definition:
//!      type-def
//!    | constant-def
//!
//! specification:
//!      definition *
//! ```

mod case_spec;
mod const_def;
mod constant;
mod declaration;
mod definition;
mod enum_body;
mod enum_type_spec;
mod specification;
mod struct_body;
mod struct_type_spec;
mod type_def;
mod type_spec;
mod union_body;
mod union_type_spec;
mod value;

use crate::{ast::Ast, error::ParseError, token::SpannedToken};

pub(crate) fn parse_tokens(
    tokens: Vec<SpannedToken<'_>>,
) -> Result<Ast, ParseError> {
    todo!()
}
