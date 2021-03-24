use geo::{Geometry as GeometryT, Point as PointT};

pub type Geometry = GeometryT<f64>;
pub type Point = PointT<f64>;

pub fn point(x: f64, y: f64) -> Point {
    Point::new(x, y)
}
