#[cfg(not(feature = "f64"))] use std::f32::consts::{E, PI};
#[cfg(feature = "f64")] use std::f64::consts::{E, PI};

use crate::{Coordinate, Node, OsmData};
use crate::Float;

const EARTH_RADIUS_KM: Float = 6378.137;

#[derive(Copy, Clone)]
pub enum Projection {
	/// https://wiki.openstreetmap.org/wiki/Web_Mercator
	WebMercator,
}


pub trait Convert {
	fn convert_to(&mut self, p: Projection);
	fn revert_from(&mut self, p: Projection);
}

impl Convert for Coordinate {
	fn convert_to(&mut self, p: Projection) {
		match p {
			Projection::WebMercator => {
				self.lat = lat2y(self.lat);
				self.lon = lon2x(self.lon);
			}
		}
	}

	fn revert_from(&mut self, p: Projection) {
		match p { 
			Projection::WebMercator => {
				self.lat = y2lat(self.lat);
				self.lon = x2lon(self.lon);
			}
		}
	}
}

impl Convert for Node {
	fn convert_to(&mut self, p: Projection) {
		self.pos.convert_to(p);
	}

	fn revert_from(&mut self, p: Projection) {
		self.pos.revert_from(p);
	}
}

impl Convert for OsmData {
	fn convert_to(&mut self, p: Projection) {
		for node in self.nodes.values_mut() {
			node.convert_to(p);
		}
	}

	fn revert_from(&mut self, p: Projection) {
		for node in self.nodes.values_mut() {
			node.revert_from(p);
		}
	}
}


pub fn lat2y(lat: Float) -> Float {
	f64::default();
	(lat.to_radians() / 2. + PI / 4.).tan().log(E) * EARTH_RADIUS_KM * 1000.
}

pub fn lon2x(lon: Float) -> Float {
	EARTH_RADIUS_KM * 1000. * lon.to_radians()
}

pub fn y2lat(y: Float) -> Float {
	(2. * (y / (EARTH_RADIUS_KM * 1000.)).exp().atan() - PI / 2.).to_degrees()
}

pub fn x2lon(x: Float) -> Float {
	(x / (EARTH_RADIUS_KM * 1000.)).to_degrees()
}


#[cfg(test)]
mod tests_convert {
	use super::*;

	#[test]
	fn projection_webmercator() {
		let original = Coordinate::new(50., 10.);

		let mut projected = original.clone();
		projected.convert_to(Projection::WebMercator);

		let mut reverted = projected.clone();
		reverted.revert_from(Projection::WebMercator);

		assert!((original.lat.abs() - reverted.lat.abs()) <= 0.000001);
		assert!((original.lon.abs() - reverted.lon.abs()) <= 0.000001);
	}
}
