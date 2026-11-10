use xdrs::{
    Encode, EncodeError, Encoder, StdEncoder,
    test_utils::{assert_encode_eq, assert_encode_err},
};

#[test]
fn common() {
    #[derive(Debug, Eq, PartialEq, Encode)]
    #[repr(i32)]
    enum Discriminant {
        Int,
        String,
        Void,
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
        Default,
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
                Manual::Default => {
                    return Err(E::Error::invalid_variant(
                        "default branch lacks an explicit discriminant value",
                    ));
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
