use crate::op::OpList;
use crate::{error::ApplyResult, geom::Geometry};
use parser::ast::{Literal, Value, ValueList};

pub mod clear;

pub struct SymInput {
    // resolved values
    pub args: Vec<Literal>,
    // feature geometry
    pub geometry: Geometry,
    // previous operations
    pub ops: OpList,
}
pub struct SymOuput {
    pub ops: OpList,
}

impl SymOuput {
    pub fn new(ops: OpList) -> Self {
        Self { ops }
    }
}

pub trait SymCommand {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput>;
}
