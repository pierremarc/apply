use geojson::Feature;
use parser::ast::{Constructor, Literal, Value};
use std::collections::HashMap;

use crate::{
    error::{ApplyError, ApplyResult},
    function::find_function,
    source::Source,
};

#[derive(Debug, Clone)]
pub struct BaseScope {
    known_values: HashMap<String, Literal>,
}

impl BaseScope {
    // the passed values shall hold all known vales when entering
    // the scope
    pub fn new(values: HashMap<String, Literal>) -> Self {
        Self {
            known_values: values,
        }
    }

    pub fn concat(&self, values: HashMap<String, Literal>) -> Self {
        let mut new_map = self.known_values.clone();
        new_map.extend(values.into_iter());
        BaseScope::new(new_map)
    }

    fn get(&self, key: &str) -> ApplyResult<Literal> {
        self.known_values
            .get(key)
            .map(|v| v.clone())
            .ok_or(ApplyError::Resolve(key.into()))
    }

    pub fn resolve(&self, value: Value) -> ApplyResult<Literal> {
        match value {
            Value::Lit(l) => Ok(l),
            Value::Fn(f) => {
                let func = find_function(&f.name)?;
                let mut args: Vec<Literal> = Vec::new();
                for arg in f.args.iter() {
                    args.push(self.resolve(arg.clone())?);
                }
                func.call(args)
            }
            Value::Data(data) => match *data.constructor {
                Constructor::Val(inner) => self.resolve(inner),
                Constructor::Select(_) => Err(ApplyError::Resolve(
                    "At this point we can't resolve a select".into(),
                )),
            },
        }
    }
}
pub struct FeatureScope<'a, S>
where
    S: Source,
{
    base: BaseScope,
    source: &'a S,
    feature: &'a Feature,
}

impl<'a, S> FeatureScope<'a, S>
where
    S: Source,
{
    pub fn new(base: BaseScope, source: &'a S, feature: &'a Feature) -> Self {
        Self {
            base,
            source,
            feature,
        }
    }

    pub fn resolve(&self, value: Value) -> ApplyResult<Literal> {
        match value {
            Value::Data(data) => match *data.constructor {
                Constructor::Select(select) => self.source.select(select, self.feature),
                Constructor::Val(inner) => self.resolve(inner),
            },
            _ => self.base.resolve(value),
        }
    }
}
