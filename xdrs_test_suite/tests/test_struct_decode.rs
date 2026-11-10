use xdrs::{Decode, Encode, StdEncoder, test_utils::assert_decode};

#[test]
fn empty() {
    #[derive(Debug, PartialEq, Decode)]
    struct Struct {}

    let input = &[];
    let exp = Struct {};

    assert_decode(input, exp);
}

#[test]
fn common() {
    #[derive(Debug, PartialEq, Decode)]
    struct Struct {
        a: i32,
        b: i64,
        c: String,
    }

    let input = {
        let mut encoder = StdEncoder::new(Vec::new());
        Encode::encode(&1i32, &mut encoder).unwrap();
        Encode::encode(&2i64, &mut encoder).unwrap();
        Encode::encode(&String::from("3"), &mut encoder).unwrap();
        encoder.into_inner()
    };
    let exp = Struct {
        a: 1,
        b: 2,
        c: "3".into(),
    };

    assert_decode(&input, exp);
}

#[test]
fn generics() {
    trait Trait {}
    impl Trait for i32 {}

    #[derive(Debug, PartialEq, Decode)]
    struct Struct<'a, A: Trait + for<'d> Decode<'d>, B: for<'d> Decode<'d>> {
        a: A,
        b: B,
        c: &'a str,
    }

    let input = {
        let mut encoder = StdEncoder::new(Vec::new());
        Encode::encode(&1i32, &mut encoder).unwrap();
        Encode::encode(&2i64, &mut encoder).unwrap();
        Encode::encode(&"3", &mut encoder).unwrap();
        encoder.into_inner()
    };
    let exp = Struct {
        a: 1i32,
        b: 2i64,
        c: "3",
    };

    assert_decode(&input, exp);
}

#[test]
fn complex() {
    #[derive(Debug, PartialEq, Decode)]
    struct StructA {
        a: i32,
        b: i64,
        c: StructB,
    }

    #[derive(Debug, PartialEq, Decode)]
    struct StructB {
        d: Vec<u8>,
        e: i64,
        f: String,
    }

    let input = {
        let mut encoder = StdEncoder::new(Vec::new());
        Encode::encode(&1i32, &mut encoder).unwrap();
        Encode::encode(&2i64, &mut encoder).unwrap();
        Encode::encode(&[0x03u8][..], &mut encoder).unwrap();
        Encode::encode(&4i64, &mut encoder).unwrap();
        Encode::encode(&"5", &mut encoder).unwrap();
        encoder.into_inner()
    };
    let exp = StructA {
        a: 1,
        b: 2,
        c: StructB {
            d: vec![0x03],
            e: 4,
            f: "5".into(),
        },
    };

    assert_decode(&input, exp);
}
