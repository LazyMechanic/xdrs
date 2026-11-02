mod utils;

use std::io::Write;

use xdrs::{Encode, Error};

#[test]
fn empty() {
    #[derive(Encode)]
    struct Derived {}

    struct Manual {}
    impl Encode for Manual {
        fn encode<W: Write>(&self, _out: &mut W) -> Result<usize, Error> {
            Ok(0)
        }
    }

    let act = Derived {};
    let exp = Manual {};

    utils::assert_pack_eq(act, exp);
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
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            let mut write = 0;
            write += self.a.encode(out)?;
            write += self.b.encode(out)?;
            write += self.c.encode(out)?;
            Ok(write)
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

    utils::assert_pack_eq(act, exp);
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
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            let mut wrote = 0;
            wrote += self.a.encode(out)?;
            wrote += self.b.encode(out)?;
            wrote += self.c.encode(out)?;
            Ok(wrote)
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

    utils::assert_pack_eq(act, exp);
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
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            let mut write = 0;
            write += self.a.encode(out)?;
            write += self.b.encode(out)?;
            write += self.c.encode(out)?;
            Ok(write)
        }
    }

    struct ManualB {
        d: Vec<u8>,
        e: i64,
        f: String,
    }
    impl Encode for ManualB {
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            let mut write = 0;
            write += self.d.encode(out)?;
            write += self.e.encode(out)?;
            write += self.f.encode(out)?;
            Ok(write)
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

    utils::assert_pack_eq(act, exp);
}
