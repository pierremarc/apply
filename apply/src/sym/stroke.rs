use parser::ast::Stroke;

use crate::{error::ApplyResult, op::stroke};

use super::{SymCommand, SymInput, SymOuput};

impl SymCommand for Stroke {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput> {
        let size = input.resolve_float(self.size.clone())?;
        let color = input.resolve_string(self.color.clone())?;
        Ok(input.concat_ops(vec![stroke(color, size)]))
    }
}
