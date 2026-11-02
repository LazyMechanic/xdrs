use xdrs::Encode;

#[derive(Encode)]
#[repr(i32)]
enum Discriminant {
    Int,
    String,
    Void,
}

#[derive(Encode)]
enum Derived {
    #[xdrs(discriminant = "Int")]
    Int(i32),
    #[xdrs(discriminant = "String")]
    String(String),
    #[xdrs(discriminant = "Void")]
    Void,
    #[xdrs(default)]
    Default,
}

fn main() {}
