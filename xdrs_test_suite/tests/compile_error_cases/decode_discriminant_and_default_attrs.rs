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
    #[xdrs(discriminant = "Discriminant::Int")]
    Int(i32),
    #[xdrs(discriminant = "Discriminant::String")]
    String(String),
    #[xdrs(discriminant = "Discriminant::Void")]
    Void,
    #[xdrs(default, discriminant = "Discriminant::Void")]
    Default,
}

fn main() {}
