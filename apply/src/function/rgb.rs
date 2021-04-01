use parser::ast::{Literal, Num};

use crate::error::{ApplyError, ApplyResult};

use super::Function;

struct RGBColor {
    r: i64,
    g: i64,
    b: i64,
}

impl RGBColor {
    fn as_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

pub struct RGB;

impl RGB {
    fn make_color(&self, args: Vec<Literal>) -> Option<RGBColor> {
        if args.len() == 3 {
            match (args.get(0), args.get(1), args.get(2)) {
                (
                    Some(Literal::Number(Num::Integer(r))),
                    Some(Literal::Number(Num::Integer(g))),
                    Some(Literal::Number(Num::Integer(b))),
                ) => Some(RGBColor {
                    r: *r,
                    g: *g,
                    b: *b,
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Function for RGB {
    fn call(&self, args: Vec<Literal>) -> ApplyResult<Literal> {
        if let Some(color) = self.make_color(args) {
            Ok(Literal::String(color.as_string()))
        } else {
            Err(ApplyError::Conversion)
        }
    }
}
