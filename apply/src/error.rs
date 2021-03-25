#[derive(Debug)]
pub enum ApplyError {
    FunctionNotFound(String),
    FunctionArg(String),
    FunctionFail(String),
    Sym(String),
    Resolve(String),
    SourceInit(String),
    Select(String),
    CommandNotFound,
    Conversion,
    Geometry,
    MissingSource,
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub type ApplyResult<T> = Result<T, ApplyError>;
