use crate::geom::{point, Point};

#[derive(Debug, Clone)]
pub enum Op {
    Text {
        text: String,
        color: String,
        x: f64,
        y: f64,
    },
    Font {
        name: String,
        size: f64,
    },
    Fill(String),
    Stroke {
        color: String,
        size: f64,
    },
    Start,
    Move(Point),
    Line(Point),
    Cubic {
        control_1: Point,
        control_2: Point,
        end: Point,
    },
    Close,
}

fn point_as_string(p: &Point) -> String {
    format!("({}, {})", p.x(), p.y())
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Text { text, color, x, y } => write!(f, "[text {} {} {} {}]", text, color, x, y),
            Op::Font { name, size } => write!(f, "[font {} {}]", name, size),
            Op::Fill(color) => write!(f, "[fill {}]", color),
            Op::Stroke { color, size } => write!(f, "[stroke {} {}]", color, size),
            Op::Start => write!(f, "[start]"),
            Op::Move(p) => write!(f, "[move {}]", point_as_string(p)),
            Op::Line(p) => write!(f, "[line {}]", point_as_string(p)),
            Op::Cubic {
                control_1,
                control_2,
                end,
            } => write!(
                f,
                "[cubic {} {} {}]",
                point_as_string(control_1),
                point_as_string(control_2),
                point_as_string(end)
            ),
            Op::Close => write!(f, "[close]"),
        }
    }
}

pub type OpList = Vec<Op>;

pub fn start() -> Op {
    Op::Start
}

pub fn move_to(x: f64, y: f64) -> Op {
    Op::Move(point(x, y))
}

pub fn line_to(x: f64, y: f64) -> Op {
    Op::Line(point(x, y))
}

pub fn close() -> Op {
    Op::Close
}

pub fn fill(color: String) -> Op {
    Op::Fill(color)
}

pub fn stroke(color: String, size: f64) -> Op {
    Op::Stroke { color, size }
}

pub fn text(text: String, color: String, x: f64, y: f64) -> Op {
    Op::Text { text, color, x, y }
}
