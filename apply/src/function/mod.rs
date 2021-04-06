use parser::ast::Literal;

use crate::error::{ApplyError, ApplyResult};

mod concat;
mod rgb;

pub trait Function {
    fn call(&self, args: Vec<Literal>) -> ApplyResult<Literal>;
}

pub fn find_function(name: &str) -> ApplyResult<Box<dyn Function>> {
    match name {
        "rgb" => Ok(Box::new(rgb::RGB)),
        "concat" => Ok(Box::new(concat::Concat)),
        _ => Err(ApplyError::FunctionNotFound(name.into())),
    }
}
