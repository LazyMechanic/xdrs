pub mod builder;

mod hir;

use crate::builder::Builder;

pub fn configure() -> Builder {
    Builder::new()
}