use parser::ast::{Literal, Value};
use std::collections::HashMap;

use crate::{
    error::{ApplyError, ApplyResult},
    function::find_function,
};

pub struct ApplyScope {
    known_values: HashMap<String, Literal>,
}

impl ApplyScope {
    // the passed values shall hold all known vales when entering
    // the scope, esp. those extracted from a source feature
    pub fn new(values: HashMap<String, Literal>) -> Self {
        Self {
            known_values: values,
        }
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
            Value::Data(d) => self.get(&d.ident),
        }
    }
}
