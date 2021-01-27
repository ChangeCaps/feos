use crate::span::*;

#[derive(Debug)]
pub enum ErrorKind {
    TypeMismatch,
    UndefinedVariable,
    UndefinedFunction,
    Unreachable,
    FunctionRedefinition,
    InvalidDerefTarget,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub code: String,
}

impl Error {
    pub fn new(kind: ErrorKind, source: impl AsRef<str>, span: Span) -> Self {
        let code = span.str_from_source(source.as_ref()).to_string();

        Self { kind, code }
    }

    pub fn from_raw(kind: ErrorKind, code: impl Into<String>) -> Self {
        Self {
            kind,
            code: code.into(),
        }
    }
}
