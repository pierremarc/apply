use geojson::{Feature, FeatureCollection, GeoJson};
use parser::ast::Num;
use std::fs::read_to_string;
use std::rc::Rc;

use crate::error::{ApplyError, ApplyResult};

use super::{Resolver, SourceT};

#[derive(Clone)]
pub struct GeoJSON {
    pub data: Rc<FeatureCollection>,
    pub srid: i64,
}

fn load_file(path: String) -> ApplyResult<FeatureCollection> {
    let json_str = read_to_string(&path)?;
    let gj = json_str.parse::<GeoJson>()?;
    match gj {
        GeoJson::FeatureCollection(fc) => Ok(fc),
        _ => Err(ApplyError::NotAFeatureCollection(path)),
    }
}

impl GeoJSON {
    pub fn init(path: String, srid: Option<Num>) -> ApplyResult<Self> {
        let srid: i64 = match srid {
            None => 4326,
            Some(num) => match num {
                Num::Integer(n) => n,
                Num::Float(n) => {
                    return Err(ApplyError::SourceInit(format!(
                        "srid should be an integer, we got a float {}",
                        n
                    )));
                }
            },
        };

        Ok(GeoJSON {
            srid,
            data: Rc::new(load_file(path)?),
        })
    }
}

impl SourceT for GeoJSON {
    fn iter(&self) -> Box<dyn Iterator<Item = &Feature> + '_> {
        Box::new(self.data.features.iter())
    }
}

impl Resolver for GeoJSON {}
