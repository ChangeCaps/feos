use crate::memory::*;
use crate::runtime::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span {
    lo: usize,
    hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Span {
            lo,
            hi,
        }
    } 
}

#[derive(Debug)]
pub enum _RuntimeError {
    UndefinedVariable,
    InvalidAssignTarget,
    InvalidDerefTarget,
    InvaildRefernce,
    InvalidMemoryID,
    InequalLeftRightHandTypes,
}

use _RuntimeError::*;

pub struct RuntimeError {
    error: _RuntimeError,
    span: Span,
}

impl RuntimeError {
    pub fn new(error: _RuntimeError, span: Span) -> Self {
        Self {
            error,
            span,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub blocks: Vec<Block>,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

///   mods    trg
/// +------+  +-+
/// |      |  | |
/// foo::bar::Baz
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Path {
    pub modules: Vec<String>,
    pub target: String,
}

impl std::fmt::Display for Path {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for module in &self.modules {
            write!(formatter, "{}::", module)?;
        }

        writeln!(formatter, "{}", self.target)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum _Statement {
    Let(String, Expression),
    Assign(Expression, Expression),
}

#[derive(Debug)]
pub struct Statement {
    pub statement: _Statement,
    pub span: Span,
}

impl Statement {
    pub fn evaluate(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<(), RuntimeError> {
        match &self.statement {
            _Statement::Let(ident, val) => {
                let val = val.evaluate(runtime, scope)?;
                let ty = val.ty();
                let id = runtime.memory.insert(val);
                scope.insert(ident, id, ty);

                Ok(())
            },
            _Statement::Assign(trg, val) => {
                let trg = trg.evaluate(runtime, scope)?;
                
                if let Value::Ref(id, ty) = &trg {
                    let val = val.evaluate(runtime, scope)?;

                    if val.ty() == *ty {
                        *runtime.memory.get_mut(id)? = val;

                        Ok(())
                    } else {
                        Err(RuntimeError::new(InequalLeftRightHandTypes, self.span))
                    }
                } else {
                    Err(RuntimeError::new(InvalidAssignTarget, self.span))
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum _Expression {
    Literal(Value),
    Variable(String),
    Dereferece(Box<Expression>),
}

#[derive(Debug)]
pub struct Expression {
    pub expression: _Expression,
    pub span: Span,
}

impl Expression {
    pub fn evaluate(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value, RuntimeError> {
        match &self.expression {
            _Expression::Literal(val) => Ok(val.clone()),
            _Expression::Variable(var) => {
                if let Some((id, ty)) = scope.get(var) {
                    Ok(Value::Ref(id.clone(), ty.clone()))
                } else {
                    Err(RuntimeError::new(UndefinedVariable, self.span))
                }
            },
            _Expression::Dereferece(reference) => {
                let reference = reference.evaluate(runtime, scope)?;

                if let Value::Ref(id, _) = &reference {
                    let val = runtime.memory.get(id)?;

                    Ok(val.clone())
                } else {
                    Err(RuntimeError::new(InvalidDerefTarget, self.span))
                }
            } 
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lalrpop_util::lalrpop_mod;

    lalrpop_mod!(grammar);

    #[test]
    fn path_parsing() {
        let path_str = "foo::bar::Baz";
        let path = Path {
            modules: vec!["foo".into(), "bar".into()],
            target: "Baz".into(),
        };

        assert_eq!(path, grammar::PathParser::new().parse(path_str).unwrap());
    }

    #[test]
    fn literal_parsing() {
        let parser = grammar::LiteralParser::new();

        assert_eq!(Value::String("foo is a bar".to_string()), parser.parse(r#""foo is a bar""#).unwrap());
        assert_eq!(Value::I32(42), parser.parse("42").unwrap());
        assert_eq!(Value::F32(420.69), parser.parse("420.69").unwrap());
    }

    #[test]
    fn expression_parsing() {
        let parser = grammar::ExprParser::new();
    }
}