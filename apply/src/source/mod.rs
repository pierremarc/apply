use geojson::Feature;
use parser::ast::{Literal, Num, Select};
use serde_json::Value;
use std::convert::TryInto;

use crate::error::{ApplyError, ApplyResult};

pub mod geojson_source;

pub trait Source: Sized {
    fn iter(&self) -> Box<dyn Iterator<Item = &Feature> + '_>;

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
}

pub fn try_literal(v: &Value) -> Option<Literal> {
    match v {
        Value::Null => Some(Literal::Nil),
        Value::Bool(b) => Some(Literal::Boolean(*b)),
        Value::String(s) => Some(Literal::String(s.clone())),
        Value::Number(n) => {
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
