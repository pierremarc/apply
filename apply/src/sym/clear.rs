use parser::ast::Clear;

use crate::error::ApplyResult;

use super::{SymCommand, SymInput, SymOuput};

impl SymCommand for Clear {
    fn exec(&self, _input: &SymInput) -> ApplyResult<SymOuput> {
        Ok(SymOuput::new(Vec::new()))
    }
}
