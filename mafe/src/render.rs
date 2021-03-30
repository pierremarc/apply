use apply::op::OpList;
use piet_common::{
    kurbo::{PathEl, Point, Shape},
    Color, Error, RenderContext,
};

fn kpoint(p: &apply::geom::Point) -> Point {
    Point { x: p.x(), y: p.y() }
}

pub fn render<Ctx>(ctx: &mut Ctx, ops: &OpList) -> Result<(), Error>
where
    Ctx: RenderContext,
{
    let mut path: Vec<PathEl> = Vec::new();
    for op in ops {
        match op {
            Op::Start => path.clear(),
            Op::Move(p) => path.push(PathEl::MoveTo(kpoint(p))),
            Op::Line(p) => path.push(PathEl::LineTo(kpoint(p))),
            Op::Cubic {
                control_1,
                control_2,
                end,
            } => path.push(PathEl::CurveTo(
                kpoint(control_1),
                kpoint(control_2),
                kpoint(end),
            )),
            Op::Close => path.push(PathEl::ClosePath),
            Op::Fill(color) => ctx.fill(path.into(), Color::from_hex_str(color)?)?,
            Op::Stroke(color, size) => {
                ctx.stroke(path.into(), Color::from_hex_str(color)?, size)?
            }
            Op::Save => ctx.save(),
            Op::Restore => ctx.restore(),
            Op::Transform((a, b, c, d, e, f)) => ctx.transform([a, b, c, d, e, f]),
            _ => {}
        }
    }

    Ok(())
}
