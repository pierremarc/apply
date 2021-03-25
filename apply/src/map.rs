use parser::ast::{Directive, MapSpec};

use crate::{
    error::{ApplyError, ApplyResult},
    layer::run_layer,
    op::OpList,
};

pub fn run_map(spec: MapSpec) -> ApplyResult<OpList> {
    let _srid = spec
        .map
        .directives
        .iter()
        .find_map(|d| match d {
            Directive::Srid(s) => Some(s.value),
            _ => None,
        })
        .ok_or(ApplyError::MissingSrid)?;
    let _extent = spec
        .map
        .directives
        .iter()
        .find_map(|d| match d {
            Directive::Extent(e) => Some(e),
            _ => None,
        })
        .ok_or(ApplyError::MissingExtent)?;

    Ok(spec
        .layers
        .iter()
        .filter_map(|layer| run_layer(layer.clone()).ok())
        .flatten()
        .collect())
}
