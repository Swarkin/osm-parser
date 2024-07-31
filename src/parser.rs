use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use serde::Deserialize;

pub type Tags = HashMap<String, String>;
pub type Nodes = HashMap<Id, Node>;
pub type Ways = HashMap<Id, Way>;
pub type Id = u64;

#[derive(Copy, Clone)]
pub enum Projection {
	/// https://wiki.openstreetmap.org/wiki/Web_Mercator
	EPSG3857,
}

#[derive(Debug, Default)]
pub struct OsmData {
	pub version: String,
	pub generator: String,
	pub copyright: String,
	pub attribution: String,
	pub license: String,
	pub bounds: Bounds,
	pub nodes: Nodes,
	pub ways: Ways,
}

impl OsmData {
	pub fn project_to(self, p: Projection) -> Self {
		match p {
			Projection::EPSG3857 => {
				Self {
					version: self.version,
					generator: self.generator,
					copyright: self.copyright,
					attribution: self.attribution,
					license: self.license,
					bounds: self.bounds,
					nodes: self.nodes.into_iter().map(|(a, b)| (a, b.project_to(p))).collect(),
					ways: self.ways,
				}
			}
		}
	}
}

#[derive(Debug, Default, Deserialize)]
pub struct Bounds {
	pub minlat: f32,
	pub minlon: f32,
	pub maxlat: f32,
	pub maxlon: f32,
}

#[derive(Debug, Default, Deserialize)]
pub struct Node {
	pub id: Id,
	pub lat: f32,
	pub lon: f32,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub tags: Option<Tags>,
}

impl Node {
	pub fn project_to(self, p: Projection) -> Self {
		match p {
			Projection::EPSG3857 => {
				Self {
					id: self.id,
					lat: self.lon / FRAC_PI_2,
					lon: -self.lat,
					timestamp: self.timestamp,
					version: self.version,
					changeset: self.changeset,
					user: self.user,
					tags: self.tags,
				}
			}
		}
	}
}

#[derive(Debug, Default, Deserialize)]
pub struct Way {
	pub id: Id,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub nodes: Vec<Id>,
	pub tags: Option<Tags>,
}

impl Way {
	pub fn tags_to_string(&self) -> String {
		if let Some(tags) = &self.tags {
			tags.iter()
				.map(|(k, v)| { format!("{k}: {v}") })
				.collect::<Vec<_>>()
				.join("\n")
		} else { String::new() }
	}
}

#[derive(Debug, Deserialize)]
struct RawData {
	version: String,
	generator: String,
	copyright: String,
	attribution: String,
	license: String,
	bounds: Bounds,
	elements: Vec<serde_json::Value>,
}

/// Parse JSON data from a .osm file aquired trough https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map.
pub fn parse(path: &str) -> Result<OsmData, Box<dyn std::error::Error>> {
	let file = std::fs::read_to_string(path).unwrap();
	let raw = serde_json::from_str::<RawData>(&file)?;
	let mut nodes = Nodes::new();
	let mut ways = Ways::new();

	for e in raw.elements {
		let t = e["type"].as_str().ok_or("\"type\" is not a string")?;
		match t {
			"node" => {
				let node = serde_json::from_value::<Node>(e)?;
				nodes.insert(node.id, node);
			}
			"way" => {
				let way = serde_json::from_value::<Way>(e)?;
				ways.insert(way.id, way);
			}
			"relation" => {
				// relations are not supported
			}
			_ => Err("invalid type")?,
		}
	}

	Ok(OsmData {
		version: raw.version,
		generator: raw.generator,
		copyright: raw.copyright,
		attribution: raw.attribution,
		license: raw.license,
		bounds: raw.bounds,
		nodes,
		ways,
	})
}
