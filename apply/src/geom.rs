use geo::{Geometry as GeometryT, Point as PointT};
use std::convert::TryInto;

// use crate::error::{ApplyError, ApplyResult};
pub type Geometry = GeometryT<f64>;
pub type Point = PointT<f64>;

pub fn point(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

pub fn from_geojson(gg: geojson::Geometry) -> Option<Geometry> {
    let x: Geometry = gg.try_into().ok()?;
    Some(x)
}
