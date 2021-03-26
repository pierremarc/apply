use crate::{
    error::ApplyError,
    op::OpList,
    source::{geojson_source::GeoJSON, Source},
    sym::make_symbology,
};
use parser::ast::{Directive, LayerBlock, Source as SourceSpec};

use crate::error::ApplyResult;

pub fn make_source(spec: SourceSpec, target_srid: i64) -> ApplyResult<Source> {
    let driver = spec.driver;
    let path = spec.path;
    let srid = spec.srid;
    match driver {
        parser::ast::Driver::Geojson => {
            Ok(Source::GeoJSON(GeoJSON::init(path, srid, target_srid)?))
        }
        _ => todo!(),
    }
}

pub fn run_layer(spec: LayerBlock, target_srid: i64) -> ApplyResult<OpList> {
    let source = spec
        .directives
        .iter()
        .find_map(|d| match d {
            Directive::Source(s) => make_source(s.clone(), target_srid).ok(),
            _ => None,
        })
        .ok_or(ApplyError::MissingSource)?;

    Ok(spec
        .directives
        .iter()
        .filter_map(|d| match d {
            Directive::Sym(s) => make_symbology(s.clone(), source.clone()).ok(),
            _ => None,
        })
        .flatten()
        .collect())
}
