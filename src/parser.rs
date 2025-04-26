use crate::types::raw::*;
use crate::types::*;

/// Parse JSON data from a string aquired trough https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map.
pub fn parse(json_string: &str) -> Result<OsmData, Box<dyn std::error::Error + Sync + Send>> {
	let raw = serde_json::from_str::<RawOsmData>(json_string)?;
	OsmData::try_from(raw)
}
