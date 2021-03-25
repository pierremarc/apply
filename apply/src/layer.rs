use crate::{
    error::ApplyError,
    op::OpList,
    source::{geojson_source::GeoJSON, Source},
    sym::make_symbology,
};
use parser::ast::{Directive, LayerBlock, Source as SourceSpec};

use crate::error::ApplyResult;

pub fn make_source(spec: SourceSpec) -> ApplyResult<Source> {
    let driver = spec.driver;
    let path = spec.path;
    let srid = spec.srid;
    match driver {
        parser::ast::Driver::Geojson => Ok(Source::GeoJSON(GeoJSON::init(path, srid)?)),
        _ => todo!(),
    }
}

// pub fn make_scope(spec: LayerBlock, parent: &BaseScope) -> ApplyResult<BaseScope> {
//     let values: HashMap<String, Literal> = spec
//         .directives
//         .iter()
//         .filter_map(|directive| match directive {
//             Directive::Data(data) => match (*data.constructor).clone() {
//                 Constructor::Val(val) => parent.resolve(val).map(|l| (data.ident.clone(), l)).ok(),
//                 _ => None,
//             },
//             _ => None,
//         })
//         .collect();

//     Ok(parent.concat(values))
// }

pub fn run_layer(spec: LayerBlock) -> ApplyResult<OpList> {
    let source = spec
        .directives
        .iter()
        .find_map(|d| match d {
            Directive::Source(s) => make_source(s.clone()).ok(),
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
