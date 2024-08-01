mod parser;
mod structs;
pub mod convert;

pub use parser::*;
pub use structs::*;

#[cfg(feature = "f64")] type Float = f64;
#[cfg(not(feature = "f64"))] type Float = f32;
