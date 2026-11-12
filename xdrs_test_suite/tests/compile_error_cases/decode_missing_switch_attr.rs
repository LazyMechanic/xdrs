use xdrs::Decode;

#[derive(Decode)]
#[repr(i32)]
enum Discriminant {
    Int,
    String,
    Void,
}

#[derive(Decode)]
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

fn main() {}
