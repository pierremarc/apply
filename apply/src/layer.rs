use crate::source::geojson_source::GeoJSON;
use crate::source::Source;
use parser::ast::Source as SourceSpec;

use crate::error::ApplyResult;

pub fn make_source(spec: SourceSpec) -> ApplyResult<impl Source> {
    let driver = spec.driver;
    let path = spec.path;
    let srid = spec.srid;
    match driver {
        parser::ast::Driver::Geojson => GeoJSON::init(path, srid),
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
