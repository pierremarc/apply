use parser::ast::Stroke;

use crate::{error::ApplyResult, op::stroke};

use super::{SymCommand, SymInput, SymOuput};

impl SymCommand for Stroke {
    fn exec(&self, input: &SymInput) -> ApplyResult<SymOuput> {
        println!("STROKE");
        let size = input.resolve_float(self.size.clone())?;
        let color = input.resolve_string(self.color.clone())?;
        println!("STROKE WITH color and size");
        Ok(input.concat_ops(vec![stroke(color, size)]))
    }
}
