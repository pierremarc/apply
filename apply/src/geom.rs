use geo::algorithm::map_coords::TryMapCoords;
use geo::{Geometry as GeometryT, Point as PointT};
use proj::Proj;
use std::convert::TryInto;

// use crate::error::{ApplyError, ApplyResult};
pub type Geometry = GeometryT<f64>;
pub type Point = PointT<f64>;

pub fn point(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn project(geom: Geometry, proj: &Proj) -> Option<Geometry> {
    geom.try_map_coords(|p| {
        let n = proj.convert(*p)?;
        Ok(n)
    })
    .ok()
}

pub fn from_geojson(gg: geojson::Geometry, proj: &Proj) -> Option<Geometry> {
    let geom: Geometry = gg.try_into().ok()?;
    project(geom, proj)
}
