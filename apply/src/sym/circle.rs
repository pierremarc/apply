// use std::convert::{TryFrom, TryInto};

use crate::{
    error::ApplyResult,
    geom::centroid,
    op::{close, line_to, move_to, start},
};
use angle::{Angle, Deg};
use parser::ast::Circle;

use super::{SymCommand, SymInput, SymOuput};

impl SymCommand for Circle {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput> {
        let center = centroid(&input.geometry)?;
        let size = input.resolve_float(self.radius.clone())?;

        let x = center.x() + size;
        let y = center.y();
        let steps = if size * 10.0 > 360.0 {
            360.0
        } else {
            size * 10.0
        };
        let mut ops = vec![start(), move_to(x, y)];
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
