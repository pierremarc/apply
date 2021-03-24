pub enum ApplyError {
    FunctionNotFound(String),
    FunctionArg(String),
    FunctionFail(String),
    Sym(String),
    Resolve(String),
}

pub type ApplyResult<T> = Result<T, ApplyError>;
