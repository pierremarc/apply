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
    MissingSrid,
    MissingExtent,
    Wrapped(Box<dyn std::error::Error>),
    NotAFeatureCollection(String),
}

impl<E> From<E> for ApplyError
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        ApplyError::Wrapped(Box::new(err))
    }
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::FunctionNotFound(desc) => write!(f, "FunctionNotFound {}", desc),
            ApplyError::FunctionArg(desc) => write!(f, "FunctionArg {}", desc),
            ApplyError::FunctionFail(desc) => write!(f, "FunctionFail {}", desc),
            ApplyError::Sym(desc) => write!(f, "Sym {}", desc),
            ApplyError::Resolve(desc) => write!(f, "Resolve {}", desc),
            ApplyError::SourceInit(desc) => write!(f, "SourceInit {}", desc),
            ApplyError::Select(desc) => write!(f, "Select {}", desc),
            ApplyError::CommandNotFound => write!(f, "CommandNotFound"),
            ApplyError::Conversion => write!(f, "Conversion"),
            ApplyError::Geometry => write!(f, "Geometry"),
            ApplyError::MissingSource => write!(f, "MissingSource"),
            ApplyError::MissingSrid => write!(f, "Missing srid in map block"),
            ApplyError::MissingExtent => write!(f, "Missing extent in map block"),
            ApplyError::Wrapped(err) => write!(f, "Other -> {}", err),
            ApplyError::NotAFeatureCollection(desc) => write!(f, "NotAFeatureCollection {}", desc),
        }
    }
}

pub type ApplyResult<T> = Result<T, ApplyError>;
