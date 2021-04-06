use parser::ast::{Literal, Num};

use crate::error::ApplyResult;

use super::Function;

pub struct Concat;

impl Function for Concat {
    fn call(&self, args: Vec<Literal>) -> ApplyResult<Literal> {
        Ok(Literal::String(
            args.iter().map(|v| format!("{}", v)).collect(),
        ))
    }
}
