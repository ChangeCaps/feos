use crate::error::*;
use crate::variant::*;

pub enum ControlFlow {
    Error(Error),
    Return(Variable),
}

impl From<Error> for ControlFlow {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}
