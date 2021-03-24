// use std::convert::{TryFrom, TryInto};

use crate::{
    error::{ApplyError, ApplyResult},
    op::{close, line_to, move_to},
};
use angle::{Angle, Deg};
use geo::{algorithm::centroid::Centroid, Geometry};
use parser::ast::{Circle, Literal, Num};

use super::{SymCommand, SymInput, SymOuput};

impl SymCommand for Circle {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput> {
        let center = match input.geometry.clone() {
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
        .ok_or(ApplyError::Geometry)?;

        let size = match input.scope.resolve(self.radius.clone())? {
            Literal::Number(n) => match n {
                Num::Float(f) => f,
                Num::Integer(i) => i as f64,
                // Num::Integer(i) => f64::try_from(i).map_err(|_| ApplyError::Conversion)?,
            },
            _ => return Err(ApplyError::FunctionArg("radius should be a number".into())),
        };

        let x = center.x() + size;
        let y = center.y();
        let steps = if size * 10.0 > 360.0 {
            360.0
        } else {
            size * 10.0
        };
        let mut ops = vec![move_to(x, y)];
        let mut step = steps;
        while step > 0.0 {
            let a = Deg(step * 360.0 / steps);
            ops.push(line_to((a.cos() * size) + x, (a.sin() * size) + y));
            step -= 1.0;
        }
        ops.push(close());

        Ok(input.concat_ops(ops))
    }
}
