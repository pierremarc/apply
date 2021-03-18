use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    Mysterious,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mysterious => write!(f, "Something bad happened..."),
        }
    }
}

pub struct Context {
    pub error: Option<ParseError>,
}
