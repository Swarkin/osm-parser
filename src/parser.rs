use crate::types::raw::RawOsmData;
use crate::OsmData;

/// Parse JSON data from a string aquired trough <https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map>.
///
/// # Errors
/// This function will return an error if `serde_json` could not parse the input string into the expected data type.
pub fn parse(json_string: &str) -> Result<OsmData, Box<dyn std::error::Error + Sync + Send>> {
	let raw = serde_json::from_str::<RawOsmData>(json_string)?;
	OsmData::try_from(raw)
}
