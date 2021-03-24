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
}

pub type ApplyResult<T> = Result<T, ApplyError>;
