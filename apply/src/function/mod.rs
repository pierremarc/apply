use parser::ast::Literal;

use crate::error::ApplyResult;

pub trait Function {
    fn call(&self, args: Vec<Literal>) -> ApplyResult<Literal>;
}

pub fn find_function(name: &str) -> ApplyResult<Box<dyn Function>> {
    todo!()
}
