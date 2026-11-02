use xdrs::Encode;

#[derive(Encode)]
union IntOrFloat {
    i: u32,
    f: f32,
}

fn main() {}
