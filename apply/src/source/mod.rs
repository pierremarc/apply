use geojson::Feature;
use geojson_source::GeoJSON;
use parser::ast::{Constructor, Literal, Num, Select, Value};
use proj::Proj;
use serde_json::Value as JsonValue;
use std::{cell::RefCell, convert::TryInto, rc::Rc};
use Value::{Data, Lit};

use crate::{
    error::{ApplyError, ApplyResult},
    function::find_function,
};

pub mod geojson_source;

pub trait SourceT {
    fn iter(&self) -> Box<dyn Iterator<Item = &Feature> + '_>;
    fn proj(&self) -> Proj;
}
pub trait Resolver: Clone {
    fn select(&self, select: Select, feature: &Feature) -> ApplyResult<Literal> {
        let props = feature
            .properties
            .clone()
            .ok_or(ApplyError::Select(format!("[GeoJSON] missing properties")))?;

        let dt = select.datatype;

        props
            .get(&select.selector)
            .and_then(try_literal)
            .and_then(|lit| match (&lit, dt) {
                (Literal::Nil, _) => Some(lit),
                (Literal::String(_), parser::ast::DataType::String) => Some(lit),
                (Literal::Number(_), parser::ast::DataType::Number) => Some(lit),
                (Literal::Boolean(_), parser::ast::DataType::Boolean) => Some(lit),
                _ => None,
            })
            .ok_or(ApplyError::Select(format!(
                "[GeoJSON] failed to get or convert property: {}",
                &select.selector
            )))
    }

    fn resolve(&self, value: Value, feature: &Feature) -> ApplyResult<Literal> {
        match value {
            Lit(l) => Ok(l),
            Value::Fn(f) => {
                let func = find_function(&f.name)?;
                let mut args: Vec<Literal> = Vec::new();
                for arg in f.args.iter() {
                    args.push(self.resolve(arg.clone(), feature)?);
                }
                func.call(args)
            }
            Data(data) => match *data.constructor {
                Constructor::Val(inner) => self.resolve(inner, feature),
                Constructor::Select(select) => self.select(select, feature),
            },
        }
    }
}

#[derive(Clone)]
pub enum Source {
    GeoJSON(GeoJSON),
}

impl SourceT for Source {
    fn iter(&self) -> Box<dyn Iterator<Item = &Feature> + '_> {
        match self {
            Source::GeoJSON(gj) => gj.iter(),
        }
    }

    fn proj(&self) -> Proj {
        match self {
            Source::GeoJSON(gj) => gj.proj(),
        }
    }
}

impl Resolver for Source {
    fn select(&self, select: Select, feature: &Feature) -> ApplyResult<Literal> {
        match self {
            Source::GeoJSON(gj) => gj.select(select, feature),
        }
    }

    fn resolve(&self, value: Value, feature: &Feature) -> ApplyResult<Literal> {
        match self {
            Source::GeoJSON(gj) => gj.resolve(value, feature),
        }
    }
}

pub type SharedSource = Rc<RefCell<dyn SourceT>>;

// pub fn new_source<S: Source>(source: S) -> SharedSource {
//     Rc::new(RefCell::new(source))
// }

pub fn try_literal(json_value: &JsonValue) -> Option<Literal> {
    match json_value {
        JsonValue::Null => Some(Literal::Nil),
        JsonValue::Bool(b) => Some(Literal::Boolean(*b)),
        JsonValue::String(s) => Some(Literal::String(s.clone())),
        JsonValue::Number(n) => {
            if n.is_i64() {
                n.as_i64().map(|n| Literal::Number(Num::Integer(n)))
            } else if n.is_u64() {
                n.as_u64()
                    .and_then(|n| n.try_into().map(|n| Literal::Number(Num::Integer(n))).ok())
            } else {
                n.as_f64().map(|n| Literal::Number(Num::Float(n)))
            }
        }
        _ => None,
    }
}
