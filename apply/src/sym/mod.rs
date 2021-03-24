use crate::{
    error::{ApplyError, ApplyResult},
    geom::{from_geojson, Geometry},
    scope::{FeatureScope, Scope},
};
use crate::{op::OpList, source::Source};
use geojson::Feature;
use parser::ast::{Command, PredGroup, Predicate, Sym};

pub mod circle;
pub mod clear;

pub struct SymInput<'a> {
    pub scope: Box<dyn Scope + 'a>,
    // feature geometry
    pub geometry: Geometry,
    // previous operations
    pub ops: OpList,
}

impl<'a> SymInput<'a> {
    pub fn new(scope: Box<dyn Scope + 'a>, geometry: Geometry, ops: OpList) -> Self {
        Self {
            scope,
            geometry,
            ops,
        }
    }

    pub fn concat_ops(&self, ops: OpList) -> SymOuput {
        let ops = [self.ops.clone(), ops].concat();
        SymOuput { ops }
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
    scope: Box<dyn Scope + 'a>,
    feature: &Feature,
    output: SymOuput,
) -> ApplyResult<SymOuput> {
    let opt_geom = feature.geometry.as_ref();
    if let Some(geom) = opt_geom.and_then(|r| from_geojson(r.clone())) {
        let input = SymInput::new(scope, geom, output.ops);
        match command {
            Command::Clear(c) => c.exec(&input),
            Command::Circle(c) => c.exec(&input),
            _ => Err(ApplyError::CommandNotFound),
        }
    } else {
        Ok(output)
    }
}

pub fn exec_consequent<'a>(
    commands: Vec<Command>,
    scope: Box<dyn Scope + 'a>,
    feature: &Feature,
) -> ApplyResult<SymOuput> {
    let init_output = Ok(SymOuput { ops: Vec::new() });

    commands.iter().fold(init_output, |acc, command| {
        acc.and_then(|output| exec_command(command, scope, feature, output))
    })
}

pub fn eval_predicate<'a>(predicate: Predicate, scope: Box<dyn Scope + 'a>) -> ApplyResult<bool> {
    match predicate {
        Predicate::Equal((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left == right)),
        Predicate::NotEqual((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left != right)),
        Predicate::GreaterThan((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left > right)),
        Predicate::GreaterThanOrEqual((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left >= right)),
        Predicate::LesserThan((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left < right)),
        Predicate::LesserThanOrEqual((left, right)) => scope
            .resolve(left)
            .and_then(|left| scope.resolve(right).map(|right| left <= right)),
    }
}

pub fn eval_predicate_group<'a>(group: PredGroup, scope: Box<dyn Scope + 'a>) -> ApplyResult<bool> {
    match group {
        PredGroup::Empty => Ok(false),
        PredGroup::Pred(predicate) => eval_predicate(predicate, scope),
        PredGroup::And { left, right } => eval_predicate_group(*left, scope)
            .and_then(|left| eval_predicate_group(*right, scope).map(|right| left && right)),
        PredGroup::Or { left, right } => eval_predicate_group(*left, scope)
            .and_then(|left| eval_predicate_group(*right, scope).map(|right| left || right)),
    }
}

pub fn make_symbology_for_feature<'a, S>(
    sym: Sym,
    source: &'a S,
    feature: &Feature,
) -> ApplyResult<OpList>
where
    S: Source,
{
    let group = sym.predicate.clone();
    let scope = Box::new(FeatureScope::new(source, feature.clone()));
    if eval_predicate_group(group, scope)? {
        let commands = sym.consequent.clone();
        let output = exec_consequent(commands, scope, feature)?;
        Ok(output.ops)
    } else {
        Ok(Vec::new())
    }
}

pub fn make_symbology<S>(sym: Sym, source: &S) -> ApplyResult<OpList>
where
    S: Source,
{
    let ops = source
        .iter()
        .filter_map(|f| make_symbology_for_feature(sym.clone(), source, f).ok())
        .flatten()
        .collect();
    Ok(ops)
}
