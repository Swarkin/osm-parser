use std::collections::HashMap;
use crate::structs::*;

pub type Id = u64;
pub type Nodes = HashMap<Id, Node>;
pub type Ways = HashMap<Id, Way>;
pub type Tags = HashMap<String, String>;


/// Parse JSON data from an .osm file aquired trough https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map.
pub fn parse(path: &str) -> Result<OsmData, Box<dyn std::error::Error>> {
	let file = std::fs::read_to_string(path).unwrap();
	let raw = serde_json::from_str::<RawOsmData>(&file)?;

	let mut nodes = Nodes::new();
	let mut ways = Ways::new();

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
			_ => Err("invalid type")?,
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
