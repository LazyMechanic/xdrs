use xdrs::{
    Decode, Encode, StdEncoder,
    test_utils::{TestSliceDecoder, assert_decode},
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
