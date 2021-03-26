use crate::{
    error::{ApplyError, ApplyResult},
    geom::{from_geojson, Geometry},
    source::{Resolver, Source},
};
use crate::{op::OpList, source::SourceT};
use geojson::Feature;
use parser::ast::{Command, Literal, PredGroup, Predicate, Sym, Value};

pub mod circle;
pub mod clear;

pub struct SymInput {
    source: Source,
    feature: Feature,
    // feature geometry
    pub geometry: Geometry,
    // previous operations
    pub ops: OpList,
}

impl SymInput {
    pub fn new(source: Source, feature: Feature, geometry: Geometry, ops: OpList) -> Self {
        Self {
            source,
            feature,
            geometry,
            ops,
        }
    }

    pub fn concat_ops(&self, ops: OpList) -> SymOuput {
        let ops = [self.ops.clone(), ops].concat();
        SymOuput { ops }
    }

    pub fn resolve(&self, value: Value) -> ApplyResult<Literal> {
        self.source.resolve(value, &self.feature)
    }
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

pub fn exec_command<'a>(
    command: &Command,
    source: Source,
    feature: &Feature,
    output: SymOuput,
) -> ApplyResult<SymOuput> {
    let opt_geom = feature.geometry.as_ref();
    let proj = source.proj();
    if let Some(geom) = opt_geom.and_then(|r| from_geojson(r.clone(), &proj)) {
        let input = SymInput::new(source, feature.clone(), geom, output.ops);
        match command {
            Command::Clear(c) => c.exec(&input),
            Command::Circle(c) => c.exec(&input),
            _ => Err(ApplyError::CommandNotFound),
        }
    } else {
        Ok(output)
    }
}

pub fn exec_consequent(
    commands: Vec<Command>,
    source: Source,
    feature: &Feature,
) -> ApplyResult<SymOuput> {
    let init_output = Ok(SymOuput { ops: Vec::new() });

    commands.iter().fold(init_output, |acc, command| {
        acc.and_then(|output| exec_command(command, source.clone(), feature, output))
    })
}

pub fn eval_predicate(
    predicate: Predicate,
    source: Source,
    feature: &Feature,
) -> ApplyResult<bool> {
    match predicate {
        Predicate::Equal((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left == right)),
        Predicate::NotEqual((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left != right)),
        Predicate::GreaterThan((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left > right)),
        Predicate::GreaterThanOrEqual((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left >= right)),
        Predicate::LesserThan((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left < right)),
        Predicate::LesserThanOrEqual((left, right)) => source
            .resolve(left, feature)
            .and_then(|left| source.resolve(right, feature).map(|right| left <= right)),
    }
}

pub fn eval_predicate_group<'a>(
    group: PredGroup,
    source: Source,
    feature: &Feature,
) -> ApplyResult<bool> {
    match group {
        PredGroup::Empty => Ok(false),
        PredGroup::Pred(predicate) => eval_predicate(predicate, source, feature),
        PredGroup::And { left, right } => eval_predicate_group(*left, source.clone(), feature)
            .and_then(|left| {
                eval_predicate_group(*right, source.clone(), feature).map(|right| left && right)
            }),
        PredGroup::Or { left, right } => eval_predicate_group(*left, source.clone(), feature)
            .and_then(|left| {
                eval_predicate_group(*right, source.clone(), feature).map(|right| left || right)
            }),
    }
}

pub fn make_symbology_for_feature(
    sym: Sym,
    source: Source,
    feature: &Feature,
) -> ApplyResult<OpList> {
    let group = sym.predicate.clone();
    if eval_predicate_group(group, source.clone(), feature)? {
        let commands = sym.consequent.clone();
        let output = exec_consequent(commands, source.clone(), feature)?;
        Ok(output.ops)
    } else {
        Ok(Vec::new())
    }
}

pub fn make_symbology(sym: Sym, source: Source) -> ApplyResult<OpList> {
    let ops = source
        .iter()
        .filter_map(|f| make_symbology_for_feature(sym.clone(), source.clone(), f).ok())
        .flatten()
        .collect();
    Ok(ops)
}
