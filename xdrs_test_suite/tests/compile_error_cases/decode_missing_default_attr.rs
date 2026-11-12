use xdrs::Decode;

#[derive(Decode)]
#[repr(i32)]
enum Discriminant {
    Int,
    String,
    Void,
}

#[derive(Decode)]
#[xdrs(switch = "Discriminant")]
enum Derived {
    #[xdrs(discriminant = "Int")]
    Int(i32),
    #[xdrs(discriminant = "String")]
    String(String),
    #[xdrs(discriminant = "Void")]
    Void,
    Default,
}

fn main() {}
