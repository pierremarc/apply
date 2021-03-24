pub enum Shape {
    Square,
    Circle,
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

use angle::{Angle, Deg};
