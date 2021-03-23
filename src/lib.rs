use angle::{Angle, Deg};
use geo::{Geometry, Point};
use serde_json::value::Value;

pub mod ast;
pub mod parser;

#[derive(Clone)]
pub enum PathIndex {
    Key(String),
    Index(usize),
}

impl From<String> for PathIndex {
    fn from(arg: String) -> Self {
        PathIndex::Key(arg)
    }
}

impl From<&str> for PathIndex {
    fn from(arg: &str) -> Self {
        PathIndex::Key(String::from(arg))
    }
}

impl From<usize> for PathIndex {
    fn from(arg: usize) -> Self {
        PathIndex::Index(arg)
    }
}

pub struct Path {
    p: Vec<PathIndex>,
}

fn append<T>(s: &Vec<T>, a: T) -> Vec<T>
where
    T: Clone,
{
    let mut t = s.clone();
    t.push(a);
    t
}

impl Path {
    pub fn new() -> Self {
        Path { p: Vec::new() }
    }

    pub fn add<I>(&self, i: I) -> Self
    where
        I: Into<PathIndex>,
    {
        Path {
            p: append(&self.p, i.into()),
        }
    }

    fn view<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        self.p.iter().fold(Some(v), |acc, i| {
            acc.and_then(|v| match i {
                PathIndex::Key(k) => v.get(k),
                PathIndex::Index(i) => v.get(i),
            })
        })
    }
}

pub trait Select {
    type Output;

    fn select_bool(&self, _value: bool) -> Option<Self::Output> {
        None
    }

    fn select_string(&self, _value: &str) -> Option<Self::Output> {
        None
    }

    fn select_u64(&self, _value: u64) -> Option<Self::Output> {
        None
    }

    fn select_i64(&self, _value: i64) -> Option<Self::Output> {
        None
    }

    fn select_f64(&self, _value: f64) -> Option<Self::Output> {
        None
    }
}

pub fn select<S>(s: S, p: &Path, data: &Value) -> Option<S::Output>
where
    S: Select,
{
    p.view(data).and_then(|value| match value {
        Value::Bool(b) => s.select_bool(*b),
        Value::String(st) => s.select_string(&st),
        Value::Number(n) => {
            if n.is_i64() {
                s.select_i64(n.as_i64().unwrap())
            } else if n.is_u64() {
                s.select_u64(n.as_u64().unwrap())
            } else {
                s.select_f64(n.as_f64().unwrap())
            }
        }
        _ => None,
    })
}

pub trait Select2<Output> {
    fn select_bool(&self, _value: bool) -> Option<Output> {
        None
    }

    fn select_string(&self, _value: &str) -> Option<Output> {
        None
    }

    fn select_u64(&self, _value: u64) -> Option<Output> {
        None
    }

    fn select_i64(&self, _value: i64) -> Option<Output> {
        None
    }

    fn select_f64(&self, _value: f64) -> Option<Output> {
        None
    }
}

pub fn select2<S, Output>(s: &S, p: &Path, data: &Value) -> Option<Output>
where
    S: Select2<Output>,
{
    p.view(data).and_then(|value| match value {
        Value::Bool(b) => s.select_bool(*b),
        Value::String(st) => s.select_string(&st),
        Value::Number(n) => {
            if n.is_i64() {
                s.select_i64(n.as_i64().unwrap())
            } else if n.is_u64() {
                s.select_u64(n.as_u64().unwrap())
            } else {
                s.select_f64(n.as_f64().unwrap())
            }
        }
        _ => None,
    })
}

pub struct SelectorString<'a, T>(Box<dyn 'a + Fn(&str) -> Option<T>>);

impl<'a, T> Select2<T> for SelectorString<'a, T> {
    fn select_string(&self, value: &str) -> Option<T> {
        (self.0)(value)
    }
}

pub fn selector_string<'a, T, F>(f: F) -> SelectorString<'a, T>
where
    F: 'a + Fn(&str) -> Option<T>,
{
    SelectorString(Box::new(f))
}

#[derive(Debug)]
pub enum Op {
    Text {
        text: String,
        font: String,
        color: String,
        size: f64,
        x: f64,
        y: f64,
    },
    Fill(String),
    Stroke {
        color: String,
        size: f64,
    },
    Move(Point<f64>),
    Line(Point<f64>),
    Close,
}

pub enum Shape {
    Square,
    Circle,
}

pub fn point(x: f64, y: f64) -> Point<f64> {
    Point::new(x, y)
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

pub fn text(text: String, font: String, color: String, size: f64, x: f64, y: f64) -> Op {
    Op::Text {
        text,
        font,
        color,
        size,
        x,
        y,
    }
}

impl Shape {
    pub fn draw(&self, center: &Point<f64>, size: f64) -> Vec<Op> {
        match self {
            Shape::Square => self.draw_square(center, size),
            Shape::Circle => self.draw_circle(center, size),
        }
    }

    pub fn draw_square(&self, center: &Point<f64>, size: f64) -> Vec<Op> {
        let x = center.x();
        let y = center.y();
        let half = size / 2.0;
        vec![
            move_to(x - half, y - half),
            line_to(x - half, y + half),
            line_to(x + half, y + half),
            line_to(x + half, y - half),
            close(),
        ]
    }
    pub fn draw_circle(&self, center: &Point<f64>, size: f64) -> Vec<Op> {
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
        ops
    }
}

pub struct MarkerSpec {
    pub shape: Shape,
    pub color: String,
    pub size: f64,
}

impl MarkerSpec {
    pub fn draw(&self, center: &Point<f64>) -> Vec<Op> {
        let mut ops = self.shape.draw(center, self.size);
        ops.push(fill(self.color.clone()));
        ops
    }
}

pub struct TextSpec {
    pub font: String,
    pub color: String,
    pub size: f64,
    pub content: String,
}

impl TextSpec {
    pub fn draw(&self, p: &Point<f64>) -> Vec<Op> {
        vec![text(
            self.content.clone(),
            self.font.clone(),
            self.color.clone(),
            self.size,
            p.x(),
            p.y(),
        )]
    }
}
pub enum Style {
    Marker(MarkerSpec),
    Text(TextSpec),
}

impl Style {
    pub fn draw(&self, geom: Geometry<f64>) -> Vec<Op> {
        match geom {
            Geometry::Point(p) => self.draw_point(p),
            _ => Vec::new(),
        }
    }

    pub fn draw_point(&self, point: Point<f64>) -> Vec<Op> {
        match self {
            Style::Marker(spec) => spec.draw(&point),
            Style::Text(spec) => spec.draw(&point),
        }
    }
}

pub mod style_fn {
    use crate::{MarkerSpec, Shape, Style, TextSpec};

    pub fn circle(color: String, size: f64) -> Style {
        Style::Marker(MarkerSpec {
            shape: Shape::Circle,
            color,
            size,
        })
    }

    pub fn square(color: String, size: f64) -> Style {
        Style::Marker(MarkerSpec {
            shape: Shape::Square,
            color,
            size,
        })
    }

    pub fn text(content: String, color: String, font: String, size: f64) -> Style {
        Style::Text(TextSpec {
            content,
            color,
            font,
            size,
        })
    }
}

pub fn parse_point(value: &Value) -> Option<Point<f64>> {
    let x = value.get("coordinates")?.get(0)?.as_f64()?;
    let y = value.get("coordinates")?.get(1)?.as_f64()?;
    Some(point(x, y))
}

#[cfg(test)]
mod apply {
    use super::*;
    use serde_json::json;
    #[test]
    fn get_value_from_path() {
        let val: serde_json::value::Value =
            json!({ "a": { "nested": true }, "b": ["an", "array"] });
        let p1 = Path::new().add("a").add("nested");
        let p2 = Path::new().add("b").add(1);
        assert_eq!(p1.view(&val), Some(&serde_json::value::Value::Bool(true)));
        assert_eq!(
            p2.view(&val),
            Some(&serde_json::value::Value::String(String::from("array")))
        );
    }
    #[test]
    fn select_against_value() {
        struct Selector;
        impl Select for Selector {
            type Output = bool;
            fn select_bool(&self, v: bool) -> Option<bool> {
                Some(!v)
            }
        }

        let val: serde_json::value::Value =
            json!({ "a": { "nested": true }, "b": ["an", "array"] });
        let p1 = Path::new().add("a").add("nested");

        assert_eq!(select(Selector, &p1, &val).unwrap(), false);
    }
    #[test]
    fn test_design() {
        let json_str = include_str!("../data/first.geojson");
        let data: Value = serde_json::from_str(json_str).unwrap();
        let features_path = Path::new().add("features");

        let _result = features_path
            .view(&data)
            .and_then(|val| val.as_array())
            .map(|features| {
                struct Selector;
                impl Select for Selector {
                    type Output = String;
                    fn select_string(&self, v: &str) -> Option<String> {
                        if v == "tree" {
                            Some(String::from("tree"))
                        } else {
                            Some(String::from("not-a-tree"))
                        }
                    }
                }
                let prop_path = Path::new().add("properties").add("symbol");
                for feature in features {
                    select(Selector, &prop_path, feature).map(|value| println!("-> {}", value));
                }
            });
    }
    #[test]
    fn test_design2() {
        let json_str = include_str!("../data/first.geojson");
        let data: Value = serde_json::from_str(json_str).unwrap();
        let features_path = Path::new().add("features");
        let selector = selector_string(|v| if v == "tree" { Some(1) } else { Some(0) });
        let prop_path = Path::new().add("properties").add("symbol");
        let mapper = move |features| {
            for feature in features {
                select2(&selector, &prop_path, feature).map(|value| println!("-> {}", value));
            }
        };

        let _result = features_path
            .view(&data)
            .and_then(|val| val.as_array())
            .map(mapper);
    }
    #[test]
    fn test_design3() {
        let json_str = include_str!("../data/first.geojson");
        let data: Value = serde_json::from_str(json_str).unwrap();
        let features_path = Path::new().add("features");
        let selector = selector_string(|v| {
            if v == "tree" {
                Some(style_fn::circle(String::from("blue"), 12.0))
            } else {
                Some(style_fn::square(String::from("red"), 6.0))
            }
        });
        let prop_path = Path::new().add("properties").add("symbol");
        let geom_path = Path::new().add("geometry");
        let mapper = move |features: &Vec<Value>| {
            features
                .iter()
                .filter_map(|feature| {
                    geom_path.view(feature).and_then(parse_point).and_then(|p| {
                        select2(&selector, &prop_path, feature).map(|style| style.draw(p.into()))
                    })
                })
                .collect::<Vec<Vec<Op>>>()
        };

        let result = features_path
            .view(&data)
            .and_then(|val| val.as_array())
            .map(mapper)
            .unwrap();

        for ops in result {
            for op in ops {
                println!(">> {:?}", op);
            }
        }
    }
}
