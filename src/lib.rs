#![allow(dead_code)]

mod parser;
pub mod types;
pub mod convert;

pub use parser::parse;
pub use types::{Coordinate, Node, Nodes, OsmData, Way, Ways};
