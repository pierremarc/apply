use crate::error::ApplyResult;

use super::{SymCommand, SymInput, SymOuput};

pub struct Clear;

impl SymCommand for Clear {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput> {
        Ok(SymOuput::new(Vec::new()))
    }
}

pub const clear: Clear = Clear;
