use xdrs::Encode;

#[derive(Encode)]
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
    #[xdrs(default, discriminant = "Discriminant::Void")]
    Default,
}

fn main() {}
