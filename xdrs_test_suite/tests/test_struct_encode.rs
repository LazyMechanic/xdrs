use xdrs::{Encode, Encoder, test_utils::assert_encode_eq};

#[test]
fn empty() {
    #[derive(Encode)]
    struct Derived {}

    struct Manual {}
    impl Encode for Manual {
        fn encode<E: Encoder>(&self, _encoder: &mut E) -> Result<(), E::Error> {
            Ok(())
        }
    }

    let act = Derived {};
    let exp = Manual {};

    assert_encode_eq(act, exp);
}

#[test]
fn common() {
    #[derive(Encode)]
    struct Derived {
        a: i32,
        b: i64,
        c: String,
    }

    struct Manual {
        a: i32,
        b: i64,
        c: String,
    }
    impl Encode for Manual {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            self.a.encode(encoder)?;
            self.b.encode(encoder)?;
            self.c.encode(encoder)?;
            Ok(())
        }
    }

    let act = Derived {
        a: 1,
        b: 2,
        c: "3".into(),
    };
    let exp = Manual {
        a: 1,
        b: 2,
        c: "3".into(),
    };

    assert_encode_eq(act, exp);
}

#[test]
fn generics() {
    trait Trait {}
    impl Trait for u32 {}

    #[derive(Encode)]
    struct Derived<'a, A: Trait + Encode, B: Encode> {
        a: A,
        b: B,
        c: &'a str,
    }

    struct Manual<'a, A: Trait + Encode, B: Encode> {
        a: A,
        b: B,
        c: &'a str,
    }
    impl<'a, A: Trait + Encode, B: Encode> Encode for Manual<'a, A, B> {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            self.a.encode(encoder)?;
            self.b.encode(encoder)?;
            self.c.encode(encoder)?;
            Ok(())
        }
    }

    let act = Derived {
        a: 1,
        b: 2,
        c: "3".into(),
    };
    let exp = Manual {
        a: 1,
        b: 2,
        c: "3".into(),
    };

    assert_encode_eq(act, exp);
}

#[test]
fn complex() {
    #[derive(Encode)]
    struct DerivedA {
        a: i32,
        b: i64,
        c: DerivedB,
    }

    #[derive(Encode)]
    struct DerivedB {
        d: Vec<u8>,
        e: i64,
        f: String,
    }

    struct ManualA {
        a: i32,
        b: i64,
        c: ManualB,
    }
    impl Encode for ManualA {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            self.a.encode(encoder)?;
            self.b.encode(encoder)?;
            self.c.encode(encoder)?;
            Ok(())
        }
    }

    struct ManualB {
        d: Vec<u8>,
        e: i64,
        f: String,
    }
    impl Encode for ManualB {
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            self.d.encode(encoder)?;
            self.e.encode(encoder)?;
            self.f.encode(encoder)?;
            Ok(())
        }
    }

    let act = DerivedA {
        a: 1,
        b: 2,
        c: DerivedB {
            d: vec![0x03],
            e: 4,
            f: "5".into(),
        },
    };
    let exp = ManualA {
        a: 1,
        b: 2,
        c: ManualB {
            d: vec![0x03],
            e: 4,
            f: "5".into(),
        },
    };

    assert_encode_eq(act, exp);
}
