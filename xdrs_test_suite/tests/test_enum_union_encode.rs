use xdrs::{
    Encode, Encoder,
    test_utils::{assert_encode_eq, assert_encode_err},
};

#[test]
fn without_default() {
    #[derive(Encode)]
    #[xdrs(switch = "i32")]
    enum Derived {
        #[xdrs(discriminant = "-1")]
        Int(i32),
        #[xdrs(discriminant = "2")]
        String(String),
        #[xdrs(discriminant = "3")]
        Void,
    }

    enum Manual {
        Int(i32),
        String(String),
        Void,
    }

    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::Int(v) => {
                    (-1i32).encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::Void => {
                    3i32.encode(encoder)?;
                    ().encode(encoder)?;
                }
            }

            Ok(())
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_encode_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_encode_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_encode_eq(act, exp);
}

#[test]
fn discriminant_enum() {
    #[derive(Debug, Eq, PartialEq, Encode)]
    #[repr(i32)]
    enum Discriminant {
        Int = -1,
        String = 2,
        Void = 3,
    }

    #[derive(Encode)]
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

    enum Manual {
        Int(i32),
        String(String),
        Void,
    }

    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::Int(v) => {
                    Discriminant::Int.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::String(v) => {
                    Discriminant::String.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::Void => {
                    Discriminant::Void.encode(encoder)?;
                    ().encode(encoder)?;
                }
            }

            Ok(())
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_encode_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_encode_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_encode_eq(act, exp);

    assert_encode_err(Derived::Default);
}

#[test]
fn discriminant_int() {
    const VOID_DISCRIMINANT: i32 = 3;

    #[derive(Encode)]
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

    enum Manual {
        Int(i32),
        String(String),
        Void,
    }

    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::Int(v) => {
                    (-1i32).encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::Void => {
                    VOID_DISCRIMINANT.encode(encoder)?;
                    ().encode(encoder)?;
                }
            }

            Ok(())
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_encode_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_encode_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_encode_eq(act, exp);

    assert_encode_err(Derived::Default);
}

#[test]
fn discriminant_uint() {
    const VOID_DISCRIMINANT: i32 = 3;

    #[derive(Encode)]
    #[xdrs(switch = "u32")]
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

    enum Manual {
        Int(i32),
        String(String),
        Void,
    }

    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::Int(v) => {
                    1u32.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::String(v) => {
                    2u32.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::Void => {
                    VOID_DISCRIMINANT.encode(encoder)?;
                    ().encode(encoder)?;
                }
            }

            Ok(())
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_encode_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_encode_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_encode_eq(act, exp);

    assert_encode_err(Derived::Default);
}

#[test]
fn discriminant_typedef() {
    type Typedef = i32;

    #[derive(Encode)]
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

    enum Manual {
        Int(i32),
        String(String),
        Void,
    }

    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::Int(v) => {
                    (-1i32).encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::String(v) => {
                    2i32.encode(encoder)?;
                    v.encode(encoder)?;
                }
                Manual::Void => {
                    3i32.encode(encoder)?;
                    ().encode(encoder)?;
                }
            }

            Ok(())
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_encode_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_encode_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_encode_eq(act, exp);

    assert_encode_err(Derived::Default);
}
