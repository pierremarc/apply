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

pub type OpList = Vec<Op>;

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
