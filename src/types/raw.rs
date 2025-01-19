use crate::types::*;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct RawBounds {
	pub minlat: f64,
	pub maxlat: f64,
	pub minlon: f64,
	pub maxlon: f64,
}

#[derive(Deserialize)]
pub struct RawNode {
	pub id: Id,
	pub lat: f64,
	pub lon: f64,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	#[serde(default)]
	pub tags: Tags,
}

#[derive(Deserialize)]
pub struct RawOsmData {
	pub version: String,
	pub generator: String,
	pub copyright: String,
	pub attribution: String,
	pub license: String,
	pub bounds: RawBounds,
	pub elements: Vec<serde_json::Value>,
}
