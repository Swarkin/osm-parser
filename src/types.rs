pub mod raw;

use raw::*;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "rustc_hash"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

#[cfg(feature = "rustc_hash")]
type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

pub type Id = u64;
pub type Nodes = HashMap<Id, Node>;
pub type Ways = HashMap<Id, Way>;
pub type Tags = HashMap<String, String>;

//region Coordinate
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
	pub lat: f64,
	pub lon: f64,
}

impl Coordinate {
	pub const ZERO: Self = Self { lat: 0.0, lon: 0.0 };
	pub const MIN: Self = Self { lat: -90.0, lon: -180.0 };
	pub const MAX: Self = Self { lat: 90.0, lon: 180.0 };
	pub const INF: Self = Self { lat: f64::INFINITY, lon: f64::INFINITY };
	pub const NEG_INF: Self = Self { lat: f64::NEG_INFINITY, lon: f64::NEG_INFINITY };

	pub const fn new(lat: f64, lon: f64) -> Self {
		Self { lat, lon }
	}
}

impl From<[f64; 2]> for Coordinate {
	fn from(value: [f64; 2]) -> Self {
		Self::new(value[0], value[1])
	}
}

impl From<(f64, f64)> for Coordinate {
	fn from(value: (f64, f64)) -> Self {
		Self::new(value.0, value.1)
	}
}
//endregion

//region Bounds
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
	pub min: Coordinate,
	pub max: Coordinate,
}

impl Bounds {
	pub const ZERO: Self = Self { min: Coordinate::ZERO, max: Coordinate::ZERO };
	pub const FULL: Self = Self { min: Coordinate::MIN, max: Coordinate::MAX };

	pub const fn new(min: Coordinate, max: Coordinate) -> Self {
		Self { min, max }
	}

	/// Calculates the exact [Bounds] by iterating trough all given [Nodes].
	pub fn calculate(nodes: &Nodes) -> Self {
		if nodes.is_empty() {
			return Self::ZERO;
		}

		let mut min = Coordinate::INF;
		let mut max = Coordinate::NEG_INF;

		for node in nodes.values() {
			min.lat = min.lat.min(node.pos.lat);
			min.lon = min.lon.min(node.pos.lon);
			max.lat = max.lat.max(node.pos.lat);
			max.lon = max.lon.max(node.pos.lon);
		}

		Self { min, max }
	}

	/// Calculates the center [Coordinate] of the current [Bounds].
	pub fn center(&self) -> Coordinate {
		Coordinate {
			lat: (self.min.lat + self.max.lat) / 2.0,
			lon: (self.min.lon + self.max.lon) / 2.0,
		}
	}
}

impl From<RawBounds> for Bounds {
	fn from(value: RawBounds) -> Self {
		Bounds {
			min: Coordinate { lat: value.minlat, lon: value.minlon },
			max: Coordinate { lat: value.maxlat, lon: value.maxlon } ,
		}
	}
}

#[cfg(test)]
mod tests_bounds {
	use super::*;

	const BOUNDS: Bounds = Bounds::new(
		Coordinate::new(41.30365, -81.90212),
		Coordinate::new(41.30453, -81.90126),
	);

	#[test]
	fn compute() {
		let mut nodes = Nodes::default();

		nodes.insert(1, Node::default().with_coordinate([41.30365, -81.90171]));
		nodes.insert(2, Node::default().with_coordinate([41.30453, -81.90169]));
		nodes.insert(3, Node::default().with_coordinate([41.30407, -81.90212]));
		nodes.insert(4, Node::default().with_coordinate([41.30407, -81.90126]));

		assert_eq!(Bounds::calculate(&nodes), BOUNDS);
	}

	#[test]
	fn center() {
		assert_eq!(BOUNDS.center(), Coordinate::new(41.30409, -81.90169));
	}
}
//endregion

//region Node
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
	pub id: Id,
	pub pos: Coordinate,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub tags: Tags,
}

impl Node {
	fn with_coordinate(self, coord: impl Into<Coordinate>) -> Self {
		Self { pos: coord.into(), ..Default::default() }
	}
}

impl From<RawNode> for Node {
	fn from(n: RawNode) -> Self {
		Self {
			id: n.id,
			pos: Coordinate { lat: n.lat, lon: n.lon },
			timestamp: n.timestamp,
			version: n.version,
			changeset: n.changeset,
			user: n.user,
			tags: n.tags,
		}
	}
}
//endregion

//region Way
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Way {
	pub id: Id,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub nodes: Vec<Id>,
	#[serde(default)]
	pub tags: Tags,
}
//endregion

//region Osm
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
	/// Calculates the exact [Bounds] by iterating trough all given [Nodes].
	pub fn calculate_bounds(&mut self) {
		self.bounds = Bounds::calculate(&self.nodes);
	}

	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty() && self.ways.is_empty()
	}
}

impl TryFrom<RawOsmData> for OsmData {
	type Error = Box<dyn std::error::Error + Sync + Send>;

	fn try_from(raw: RawOsmData) -> Result<Self, Self::Error> {
		let mut nodes = Nodes::default();
		let mut ways = Ways::default();

		for e in raw.elements {
			let t = e["type"].as_str().ok_or("\"type\" is not a string")?;
			match t {
				"node" => {
					let node = serde_json::from_value::<RawNode>(e)?;
					nodes.insert(node.id, node.into());
				}
				"way" => {
					let way = serde_json::from_value::<Way>(e)?;
					ways.insert(way.id, way);
				}
				"relation" => {
					// relations are not supported
				}
				_ => Err("invalid element type")?,
			}
		}

		Ok(OsmData {
			version: raw.version,
			generator: raw.generator,
			copyright: raw.copyright,
			attribution: raw.attribution,
			license: raw.license,
			bounds: raw.bounds.into(),
			nodes,
			ways,
		})
	}
}
//endregion

//region Tags
pub fn merge_tags(to: &mut Tags, from: Tags) {
	to.extend(from);
}

#[cfg(test)]
mod tests_tags {
	use super::*;

	#[test]
	fn merge() {
		let mut from = Tags::default();
		from.insert(String::from("1"), String::from("2"));

		let mut to = Tags::default();
		to.insert(String::from("1"), String::from("3"));

		merge_tags(&mut to, from);

		let mut expected = Tags::default();
		expected.insert(String::from("1"), String::from("2"));

		assert_eq!(to, expected);
	}
}
//endregion
