use apply::op::{Op, OpList};
use piet::{
    kurbo::{Affine, PathEl, Point},
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
            Op::Fill(color) => {
                let brush = Color::from_hex_str(color).map_err(|_| Error::InvalidInput)?;
                ctx.fill(path.as_slice(), &brush);
            }
            Op::Stroke { color, size } => {
                let brush = Color::from_hex_str(color).map_err(|_| Error::InvalidInput)?;
                ctx.stroke(path.as_slice(), &brush, *size)
            }
            Op::Save => {
                ctx.save().unwrap();
            }
            Op::Restore => {
                ctx.restore().unwrap();
            }
            Op::Transform((a, b, c, d, e, f)) => {
                ctx.transform(Affine::new([*a, *b, *c, *d, *e, *f]))
            }
            _ => {}
        }
    }

    Ok(())
}
