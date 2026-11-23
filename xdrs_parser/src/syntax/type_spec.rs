use winnow::{
    ModalResult, Parser,
    combinator::{alt, opt},
    seq,
};

use crate::{
    syntax::{
        enum_type_spec::EnumTypeSpec, ident::Ident,
        struct_type_spec::StructTypeSpec, token::Token,
        union_type_spec::UnionTypeSpec,
    },
    token::TokenStream,
};

/// ```text
/// type-specifier:
///      [ "unsigned" ] "int"
///    | [ "unsigned" ] "hyper"
///    | "float"
///    | "double"
///    | "quadruple"
///    | "bool"
///    | enum-type-spec
///    | struct-type-spec
///    | union-type-spec
///    | identifier
/// ```
#[allow(non_snake_case)]
pub fn TypeSpec<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeSpec<'t>> {
    alt((
        alt((
            Int.map(TypeSpec::Int),
            Hyper.map(TypeSpec::Hyper),
            Float.map(TypeSpec::Float),
            Double.map(TypeSpec::Double),
            Quadruple.map(TypeSpec::Quadruple),
            Bool.map(TypeSpec::Bool),
        )),
        alt((
            EnumTypeSpec.map(TypeSpec::EnumTypeSpec),
            StructTypeSpec.map(TypeSpec::StructTypeSpec),
            UnionTypeSpec.map(TypeSpec::UnionTypeSpec),
            Ident.map(TypeSpec::Ident),
        )),
    ))
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Int<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Int> {
    seq! {
        Int {
            unsigned: opt(Token![unsigned]),
            int: Token![int],
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Hyper<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Hyper> {
    seq! {
        Hyper {
            unsigned: opt(Token![unsigned]),
            hyper: Token![hyper],
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Float<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Float> {
    seq! {
        Float {
            float: Token![float],
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Double<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Double> {
    seq! {
        Double {
            double: Token![double],
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Quadruple<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Quadruple> {
    seq! {
        Quadruple {
            quadruple: Token![quadruple],
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn Bool<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Bool> {
    seq! {
        Bool {
            bool: Token![bool],
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeSpec<'t> {
    Int(Int),
    Hyper(Hyper),
    Float(Float),
    Double(Double),
    Quadruple(Quadruple),
    Bool(Bool),
    EnumTypeSpec(EnumTypeSpec<'t>),
    StructTypeSpec(StructTypeSpec<'t>),
    UnionTypeSpec(UnionTypeSpec<'t>),
    Ident(Ident<'t>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Int {
    pub unsigned: Option<Token![unsigned]>,
    pub int: Token![int],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Hyper {
    pub unsigned: Option<Token![unsigned]>,
    pub hyper: Token![hyper],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Float {
    pub float: Token![float],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Double {
    pub double: Token![double],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Quadruple {
    pub quadruple: Token![quadruple],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bool {
    pub bool: Token![bool],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type Int;
        pub fn int_asserts(acc) {
            asserts! {
                acc;
                type Int;
                "int": Int { unsigned: None, .. },
                "unsigned int": Int { unsigned: Some(_), .. },
            }
        }
    }

    define_asserts! {
        type Hyper;
        pub fn hyper_asserts(acc) {
            asserts! {
                acc;
                type Hyper;
                "hyper": Hyper { unsigned: None, .. },
                "unsigned hyper": Hyper { unsigned: Some(_), .. },
            }
        }
    }

    define_asserts! {
        type Float;
        pub fn float_asserts(acc) {
            asserts! {
                acc;
                type Float;
                "float": Float { .. },
            }
        }
    }

    define_asserts! {
        type Double;
        pub fn double_asserts(acc) {
            asserts! {
                acc;
                type Double;
                "double": Double { .. },
            }
        }
    }

    define_asserts! {
        type Quadruple;
        pub fn quadruple_asserts(acc) {
            asserts! {
                acc;
                type Quadruple;
                "quadruple": Quadruple { .. },
            }
        }
    }

    define_asserts! {
        type Bool;
        pub fn bool_asserts(acc) {
            asserts! {
                acc;
                type Bool;
                "bool": Bool { .. },
            }
        }
    }

    define_asserts! {
        type TypeSpec;
        pub fn type_spec_asserts(acc) {
            for (input, assert) in int_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Int(v) => assert(v) }
            }
            for (input, assert) in hyper_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Hyper(v) => assert(v) }
            }
            for (input, assert) in float_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Float(v) => assert(v) }
            }
            for (input, assert) in double_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Double(v) => assert(v) }
            }
            for (input, assert) in quadruple_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Quadruple(v) => assert(v) }
            }
            for (input, assert) in bool_asserts() {
                asserts! { acc; type TypeSpec; { input.clone() }: TypeSpec::Bool(v) => assert(v) }
            }
        }
    }

    #[test]
    fn int() {
        for (input, assert) in int_asserts() {
            assert(&parse_str(Int, input).unwrap());
        }
    }

    #[test]
    fn hyper() {
        for (input, assert) in hyper_asserts() {
            assert(&parse_str(Hyper, input).unwrap());
        }
    }

    #[test]
    fn float() {
        for (input, assert) in float_asserts() {
            assert(&parse_str(Float, input).unwrap());
        }
    }

    #[test]
    fn double() {
        for (input, assert) in double_asserts() {
            assert(&parse_str(Double, input).unwrap());
        }
    }

    #[test]
    fn quadruple() {
        for (input, assert) in quadruple_asserts() {
            assert(&parse_str(Quadruple, input).unwrap());
        }
    }

    #[test]
    fn boolean() {
        for (input, assert) in bool_asserts() {
            assert(&parse_str(Bool, input).unwrap());
        }
    }

    #[test]
    fn type_spec() {
        for (input, assert) in type_spec_asserts() {
            assert(&parse_str(TypeSpec, input).unwrap());
        }
    }
}
