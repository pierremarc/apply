use parser::ast::{Literal, Value};
use std::collections::HashMap;

use crate::{error::ApplyResult, function::find_function};

pub struct ApplyScope {
    known_values: HashMap<String, Literal>,
}

impl ApplyScope {
    pub fn new(values: HashMap<String, Literal>) -> Self {
        Self {
            known_values: values,
        }
    }

    pub fn get(&self, key: &str) -> Option<Literal> {
        self.known_values.get(key).map(|v| v.clone())
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
            Value::Data(d) => todo!(),
        }
    }
}
