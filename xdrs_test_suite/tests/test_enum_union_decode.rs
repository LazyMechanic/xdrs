use xdrs::{
    Decode, Encode, Encoder, StdEncoder,
    test_utils::{TestSliceDecoder, assert_decode, assert_decode_eq},
};

#[test]
fn without_default() {
    #[derive(Debug, PartialEq, Decode)]
    #[xdrs(switch = "i32")]
    enum Enum {
        #[xdrs(discriminant = "1")]
        VariantA(i32),
        #[xdrs(discriminant = "2")]
        VariantB(i64),
        #[xdrs(discriminant = "3")]
        VariantC(String),
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&1i32, &mut encoder).unwrap();
            Encode::encode(&2i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantA(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&2i32, &mut encoder).unwrap();
            Encode::encode(&2i64, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantB(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&3i32, &mut encoder).unwrap();
            Encode::encode(&"hello world", &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantC(String::from("hello world"));

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&4i32, &mut encoder).unwrap();
            Encode::encode(&2i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let mut decoder = TestSliceDecoder::new(&input);

        assert!(Enum::decode(&mut decoder).is_err());
    }
}

#[test]
fn with_default_void() {
    #[derive(Debug, PartialEq, Decode)]
    #[xdrs(switch = "i32")]
    enum Enum {
        #[xdrs(discriminant = "1")]
        VariantA(i32),
        #[xdrs(discriminant = "2")]
        VariantB(i64),
        #[xdrs(discriminant = "3")]
        VariantC(String),
        #[xdrs(default)]
        Default,
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&1i32, &mut encoder).unwrap();
            Encode::encode(&2i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantA(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&2i32, &mut encoder).unwrap();
            Encode::encode(&2i64, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantB(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&3i32, &mut encoder).unwrap();
            Encode::encode(&"hello world", &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantC(String::from("hello world"));

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&4i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::Default;

        assert_decode(&input, exp);
    }
}

#[test]
fn with_default_typed() {
    #[derive(Debug, PartialEq, Decode)]
    #[xdrs(switch = "i32")]
    enum Enum {
        #[xdrs(discriminant = "1")]
        VariantA(i32),
        #[xdrs(discriminant = "2")]
        VariantB(i64),
        #[xdrs(discriminant = "3")]
        VariantC(String),
        #[xdrs(default)]
        Default(i32),
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&1i32, &mut encoder).unwrap();
            Encode::encode(&2i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantA(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&2i32, &mut encoder).unwrap();
            Encode::encode(&2i64, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantB(2);

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&3i32, &mut encoder).unwrap();
            Encode::encode(&"hello world", &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::VariantC(String::from("hello world"));

        assert_decode(&input, exp);
    }

    {
        let input = {
            let mut encoder = StdEncoder::new(Vec::new());
            Encode::encode(&4i32, &mut encoder).unwrap();
            Encode::encode(&42i32, &mut encoder).unwrap();
            encoder.into_inner()
        };
        let exp = Enum::Default(42);

        assert_decode(&input, exp);
    }
}

#[test]
fn discriminant_enum() {
    #[derive(Debug, Eq, PartialEq, Encode, Decode)]
    #[repr(i32)]
    enum Discriminant {
        Int = -1,
        String = 2,
        Void = 3,
    }

    #[derive(Debug, Eq, PartialEq, Decode)]
    #[xdrs(switch = "Discriminant")]
    enum Derived {
        #[xdrs(discriminant = "Discriminant::Int")]
        Int(i32),
        #[xdrs(discriminant = "Discriminant::String")]
        String(String),
        #[xdrs(discriminant = "Discriminant::Void")]
        Void,
        #[xdrs(default)]
        Default,
    }

    enum Reference {
        Int(i32),
        String(String),
        Void,
        Default,
    }

    impl Encode for Reference {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Reference::Int(v) => {
                    Discriminant::Int.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::String(v) => {
                    Discriminant::String.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::Void => {
                    Discriminant::Void.encode(encoder)?;
                    Ok(())
                }
                Reference::Default => {
                    16315341i32.encode(encoder)?;
                    Ok(())
                }
            }
        }
    }

    let refc = Reference::Int(123);
    let exp = Derived::Int(123);
    assert_decode_eq(refc, exp);

    let refc = Reference::String("hello world".into());
    let exp = Derived::String("hello world".into());
    assert_decode_eq(refc, exp);

    let refc = Reference::Void;
    let exp = Derived::Void;
    assert_decode_eq(refc, exp);

    let refc = Reference::Default;
    let exp = Derived::Default;
    assert_decode_eq(refc, exp);
}

#[test]
fn discriminant_i32() {
    const VOID_DISCRIMINANT: i32 = 3;

    #[derive(Debug, Eq, PartialEq, Decode)]
    #[xdrs(switch = "i32")]
    enum Derived {
        #[xdrs(discriminant = "-1")]
        Int(i32),
        #[xdrs(discriminant = "2")]
        String(String),
        #[xdrs(discriminant = "VOID_DISCRIMINANT")]
        Void,
        #[xdrs(default)]
        Default,
    }

    enum Reference {
        Int(i32),
        String(String),
        Void,
        Default,
    }

    impl Encode for Reference {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Reference::Int(v) => {
                    (-1i32).encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::Void => {
                    VOID_DISCRIMINANT.encode(encoder)?;
                    Ok(())
                }
                Reference::Default => {
                    16315341i32.encode(encoder)?;
                    Ok(())
                }
            }
        }
    }

    let refc = Reference::Int(123);
    let exp = Derived::Int(123);
    assert_decode_eq(refc, exp);

    let refc = Reference::String("hello world".into());
    let exp = Derived::String("hello world".into());
    assert_decode_eq(refc, exp);

    let refc = Reference::Void;
    let exp = Derived::Void;
    assert_decode_eq(refc, exp);

    let refc = Reference::Default;
    let exp = Derived::Default;
    assert_decode_eq(refc, exp);
}

#[test]
fn discriminant_u32() {
    const VOID_DISCRIMINANT: u32 = 3;

    #[derive(Debug, Eq, PartialEq, Decode)]
    #[xdrs(switch = "i32")]
    enum Derived {
        #[xdrs(discriminant = "1")]
        Int(i32),
        #[xdrs(discriminant = "2")]
        String(String),
        #[xdrs(discriminant = "VOID_DISCRIMINANT")]
        Void,
        #[xdrs(default)]
        Default,
    }

    enum Reference {
        Int(i32),
        String(String),
        Void,
        Default,
    }

    impl Encode for Reference {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Reference::Int(v) => {
                    1u32.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::Void => {
                    VOID_DISCRIMINANT.encode(encoder)?;
                    Ok(())
                }
                Reference::Default => {
                    16315341i32.encode(encoder)?;
                    Ok(())
                }
            }
        }
    }

    let refc = Reference::Int(123);
    let exp = Derived::Int(123);
    assert_decode_eq(refc, exp);

    let refc = Reference::String("hello world".into());
    let exp = Derived::String("hello world".into());
    assert_decode_eq(refc, exp);

    let refc = Reference::Void;
    let exp = Derived::Void;
    assert_decode_eq(refc, exp);

    let refc = Reference::Default;
    let exp = Derived::Default;
    assert_decode_eq(refc, exp);
}

#[test]
fn discriminant_typedef() {
    type Typedef = i32;

    #[derive(Debug, Eq, PartialEq, Decode)]
    #[xdrs(switch = "Typedef")]
    enum Derived {
        #[xdrs(discriminant = "-1")]
        Int(i32),
        #[xdrs(discriminant = "2")]
        String(String),
        #[xdrs(discriminant = "3")]
        Void,
        #[xdrs(default)]
        Default,
    }

    enum Reference {
        Int(i32),
        String(String),
        Void,
        Default,
    }

    impl Encode for Reference {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Reference::Int(v) => {
                    (-1i32).encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                    Ok(())
                }
                Reference::Void => {
                    3.encode(encoder)?;
                    Ok(())
                }
                Reference::Default => {
                    16315341i32.encode(encoder)?;
                    Ok(())
                }
            }
        }
    }

    let refc = Reference::Int(123);
    let exp = Derived::Int(123);
    assert_decode_eq(refc, exp);

    let refc = Reference::String("hello world".into());
    let exp = Derived::String("hello world".into());
    assert_decode_eq(refc, exp);

    let refc = Reference::Void;
    let exp = Derived::Void;
    assert_decode_eq(refc, exp);

    let refc = Reference::Default;
    let exp = Derived::Default;
    assert_decode_eq(refc, exp);
}
