use serde::Deserialize;
use crate::parser::{Id, Nodes, Ways, Tags};


#[derive(Copy, Clone)]
pub enum Projection {
	/// https://wiki.openstreetmap.org/wiki/Web_Mercator
	EPSG3857,
}


//region Coordinate
#[derive(Debug, Default, PartialEq)]
pub struct Coordinate {
	pub lat: f32,
	pub lon: f32,
}

impl Coordinate {
	pub const ZERO: Self = Self { lat: 0.0, lon: 0.0 };
	pub const MIN: Self = Self { lat: -90.0, lon: -180.0 };
	pub const MAX: Self = Self { lat: 90.0, lon: 180.0 };

	pub const fn new(lat: f32, lon: f32) -> Self {
		Self { lat, lon }
	}

	pub fn project_to(&mut self, p: Projection) {
		use std::f32::consts::FRAC_PI_2;
		
		match p {
			Projection::EPSG3857 => {
				let v = self.lat;
				self.lat = self.lon / FRAC_PI_2;
				self.lon = -v;
			}
		}
	}
}

#[cfg(test)]
mod tests_coordinate {
	use super::*;

	#[test]
	fn project_to_epsg3857() {
		let mut coords = Coordinate::new(41.304, -81.9017);
		coords.project_to(Projection::EPSG3857);

		assert_eq!(coords, Coordinate::new(-52.140244, -41.304));
	}
}
//endregion

//region Bounds
#[derive(Debug, Default, PartialEq)]
pub struct Bounds {
	pub min: Coordinate,
	pub max: Coordinate,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawBounds {
	pub minlat: f32,
	pub maxlat: f32,
	pub minlon: f32,
	pub maxlon: f32,
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

	/// Computes the exact [Bounds] by looping trough the given [Nodes].
	pub fn compute(nodes: &Nodes) -> Self {
		if nodes.is_empty() {
			return Bounds::ZERO;
		}

		let mut min = Coordinate::MAX;
		let mut max = Coordinate::MIN;

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
			lat: self.min.lat + (self.max.lat - self.min.lat),
			lon: self.min.lon + (self.max.lon - self.min.lon),
		}
	}
}

#[cfg(test)]
mod tests_bounds {
	use super::*;

	#[test]
	fn compute() {
		let nodes = Nodes::from([
			(1, Node::from_coordinate(Coordinate::MIN)),
			(2, Node::from_coordinate(Coordinate::MAX)),
		]);

		let bounds = Bounds::compute(&nodes);
		assert_eq!(bounds, Bounds::FULL);
	}
}
//endregion

//region Node
#[derive(Debug)]
pub struct Node {
	pub id: Id,
	pub pos: Coordinate,
	pub timestamp: String,
	pub version: u32,
	pub changeset: u64,
	pub user: String,
	pub tags: Option<Tags>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawNode {
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

	pub fn project_to(&mut self, p: Projection) {
		self.pos.project_to(p);
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
//endregion

//region Osm
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

#[derive(Debug, Deserialize)]
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
	pub fn project_to(&mut self, p: Projection) {
		for node in self.nodes.values_mut() {
			node.project_to(p);
		}
	}

	/// Computes the exact Bounds by looping trough every Node.
	pub fn compute_bounds(&mut self) {
		if self.nodes.is_empty() {
			self.bounds = Bounds::default();
		}

		self.bounds = Bounds::compute(&self.nodes);
	}
}
//endregion
