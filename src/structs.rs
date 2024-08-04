use serde::Deserialize;

use crate::Float;
use crate::parser::{Id, Nodes, Tags, Ways};

//region Coordinate
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Coordinate {
	pub lat: Float,
	pub lon: Float,
}

impl Coordinate {
	pub const ZERO: Self = Self { lat: 0.0, lon: 0.0 };
	pub const MIN: Self = Self { lat: -90.0, lon: -180.0 };
	pub const MAX: Self = Self { lat: 90.0, lon: 180.0 };
	pub const INF: Self = Self { lat: Float::INFINITY, lon: Float::INFINITY };
	pub const NEG_INF: Self = Self { lat: Float::NEG_INFINITY, lon: Float::NEG_INFINITY };

	pub const fn new(lat: Float, lon: Float) -> Self {
		Self { lat, lon }
	}
}
//endregion

//region Bounds
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Bounds {
	pub min: Coordinate,
	pub max: Coordinate,
}

#[derive(Default, Deserialize)]
pub(crate) struct RawBounds {
	pub minlat: Float,
	pub maxlat: Float,
	pub minlon: Float,
	pub maxlon: Float,
}

impl From<RawBounds> for Bounds {
	fn from(value: RawBounds) -> Self {
		Bounds {
			min: Coordinate::new(value.minlat, value.minlon),
			max: Coordinate::new(value.maxlat, value.maxlon),
		}
	}
}

impl Bounds {
	pub const ZERO: Self = Self { min: Coordinate::ZERO, max: Coordinate::ZERO };
	pub const FULL: Self = Self { min: Coordinate::MIN, max: Coordinate::MAX };

	pub const fn new(min: Coordinate, max: Coordinate) -> Self {
		Self { min, max }
	}

	/// Computes the exact [Bounds] by looping trough the given [Nodes].
	pub fn compute(nodes: &Nodes) -> Self {
		if nodes.is_empty() {
			return Bounds::ZERO;
		}

		let mut max = Coordinate::NEG_INF;
		let mut min = Coordinate::INF;

		for node in nodes.values() {
			min.lat = min.lat.min(node.pos.lat);
			max.lat = max.lat.max(node.pos.lat);
			min.lon = min.lon.min(node.pos.lon);
			max.lon = max.lon.max(node.pos.lon);
		}

		Bounds { min, max }
	}

	/// Calculate the center of the current bounds.
	pub fn center(&self) -> Coordinate {
		Coordinate {
			lat: (self.min.lat + self.max.lat) / 2.0,
			lon: (self.min.lon + self.max.lon) / 2.0,
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
		let nodes = Nodes::from([
			(1, Node::from_coordinate(Coordinate::new(41.30365, -81.90171))),
			(2, Node::from_coordinate(Coordinate::new(41.30453, -81.90169))),
			(3, Node::from_coordinate(Coordinate::new(41.30407, -81.90212))),
			(4, Node::from_coordinate(Coordinate::new(41.30407, -81.90126))),
		]);

		assert_eq!(Bounds::compute(&nodes), BOUNDS);
	}

	#[test]
	fn center() {
		#[cfg(feature = "f64")]
		assert_eq!(BOUNDS.center(), Coordinate::new(41.30409, -81.90169));
		#[cfg(not(feature = "f64"))]
		assert_eq!(BOUNDS.center(), Coordinate::new(41.304092, -81.90169));
	}
}
//endregion

//region Node
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
	pub id: Id,
	pub pos: Coordinate,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub tags: Option<Tags>,
}

#[derive(Deserialize)]
pub(crate) struct RawNode {
	pub id: Id,
	pub lat: Float,
	pub lon: Float,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub tags: Option<Tags>,
}

impl Node {
	pub const fn default_const() -> Self {
		Self {
			id: 0,
			pos: Coordinate::ZERO,
			timestamp: String::new(),
			version: 0,
			changeset: 0,
			user: String::new(),
			tags: None,
		}
	}

	pub const fn from_coordinate(coords: Coordinate) -> Self {
		let mut node = Node::default_const();
		node.pos = coords;
		node
	}
}

impl Default for Node {
	fn default() -> Self {
		Self::default_const()
	}
}

impl From<RawNode> for Node {
	fn from(value: RawNode) -> Self {
		Self {
			id: value.id,
			pos: Coordinate::new(value.lat, value.lon),
			timestamp: value.timestamp,
			version: value.version,
			changeset: value.changeset,
			user: value.user,
			tags: value.tags,
		}
	}
}
//endregion

//region Way
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
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
	#[deprecated]
	pub fn tags_to_string(&self) -> String {
		if let Some(tags) = &self.tags {
			tags.iter()
				.map(|(k, v)| { format!("{k}: {v}") })
				.collect::<Vec<_>>()
				.join("\n")
		} else { String::new() }
	}
}
//endregion

//region Osm
#[derive(Debug, Default, Clone, PartialEq)]
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

#[derive(Deserialize)]
pub(crate) struct RawOsmData {
	pub version: String,
	pub generator: String,
	pub copyright: String,
	pub attribution: String,
	pub license: String,
	pub bounds: RawBounds,
	pub elements: Vec<serde_json::Value>,
}

impl OsmData {
	/// Computes the exact Bounds by looping trough every Node.
	pub fn compute_bounds(&mut self) {
		self.bounds = Bounds::compute(&self.nodes);
	}
}
//endregion
