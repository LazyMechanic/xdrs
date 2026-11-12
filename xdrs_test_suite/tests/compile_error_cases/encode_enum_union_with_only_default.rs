use xdrs::Encode;

#[derive(Encode)]
#[xdrs(switch = "i32")]
enum Enum {
    #[xdrs(default)]
    Default,
}

fn main() {}
