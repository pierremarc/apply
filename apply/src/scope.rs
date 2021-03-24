use std::rc::Rc;

use geojson::Feature;
use parser::ast::{Constructor, Literal, Value};

use crate::{
    error::{ApplyError, ApplyResult},
    function::find_function,
    source::Source,
};

pub trait Scope {
    fn resolve(&self, value: Value) -> ApplyResult<Literal> {
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

#[derive(Debug, Clone)]
pub struct BaseScope;

impl Scope for BaseScope {}

#[derive(Debug, Clone)]
pub struct FeatureScope<S>
where
    S: Source,
{
    source: Rc<S>,
    feature: Feature,
}

impl<S> FeatureScope<S>
where
    S: Source,
{
    pub fn new<'a>(source: &S, feature: Feature) -> Self {
        Self {
            source: Rc::new(*source),
            feature,
        }
    }
}

impl<S> Scope for FeatureScope<S>
where
    S: Source,
{
    fn resolve(&self, value: Value) -> ApplyResult<Literal> {
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
                Constructor::Select(select) => self.source.select(select, &self.feature),
            },
        }
    }
}
