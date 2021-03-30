use geo::algorithm::centroid::Centroid;
use geo::algorithm::map_coords::TryMapCoords;
use geo::{Geometry as GeometryT, Point as PointT};
use proj::Proj;
use std::convert::TryInto;

use crate::error::{ApplyError, ApplyResult};

// use crate::error::{ApplyError, ApplyResult};
pub type Geometry = GeometryT<f64>;
pub type Point = PointT<f64>;

pub type Mat = (f64, f64, f64, f64, f64, f64);

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

pub fn centroid(geom: &Geometry) -> ApplyResult<Point> {
    match geom {
        Geometry::Point(g) => Some(g.centroid()),
        Geometry::Line(g) => Some(g.centroid()),
        Geometry::Rect(g) => Some(g.centroid()),
        Geometry::LineString(g) => g.centroid(),
        Geometry::Polygon(g) => g.centroid(),
        Geometry::MultiPoint(g) => g.centroid(),
        Geometry::MultiLineString(g) => g.centroid(),
        Geometry::MultiPolygon(g) => g.centroid(),

        _ => todo!(),
    }
    .ok_or(ApplyError::Geometry)
}
