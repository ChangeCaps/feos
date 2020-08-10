use crate::memory::*;
use crate::runtime::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span {
    lo: usize,
    hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Span { lo, hi }
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeError {
    UndefinedVariable,
    UndefinedBlock,
    InvalidAssignTarget,
    InvalidDerefTarget,
    InvalidReferenceTarget,
    InvaildReference,
    InvalidMemoryID,
    InequalLeftRightHandTypes,
}

use RuntimeError::*;

#[derive(Clone, Debug)]
pub struct SpannedRuntimeError {
    error: RuntimeError,
    span: Option<Span>,
}

impl SpannedRuntimeError {
    pub fn new(error: RuntimeError) -> Self {
        Self { error, span: None }
    }

    pub fn with_span(error: RuntimeError, span: Span) -> Self {
        Self {
            error,
            span: Some(span),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub program_block: ProgramBlock,
}

impl Program {
    pub fn new(source: &str) -> Result<Self, String> {
        let program_block = crate::grammar::ProgramBlockParser::new()
            .parse(source)
            .map_err(|err| format!("{}", err))?;

        Ok(Self { program_block })
    }

    pub fn run(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<(), SpannedRuntimeError> {
        for statement in &self.program_block.statements {
            statement.evaluate(self, runtime, scope)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ProgramBlock {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}

impl Block {
    pub fn evaluate(
        &self,
        program: &Program,
        runtime: &mut Runtime,
        scope: &mut Scope,
    ) -> Result<ControlFlow, SpannedRuntimeError> {
        for statement in &self.statements {
            match statement.evaluate(program, runtime, scope)? {
                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                ControlFlow::None(_) => (),
            }
        }

        if let Some(expression) = &self.expression {
            expression.evaluate(program, runtime, scope)
        } else {
            Ok(ControlFlow::None(Value::Null))
        }
    }
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
pub enum ControlFlow {
    Return(Value),
    None(Value),
}

#[derive(Debug)]
pub enum _Statement {
    Let(String, Expression),
    Assign(Expression, Expression),
    Expression(Expression),
    Block(Block),
}

#[derive(Debug)]
pub struct Statement {
    pub statement: _Statement,
    pub span: Span,
}

impl Statement {
    pub fn evaluate(
        &self,
        program: &Program,
        runtime: &mut Runtime,
        scope: &mut Scope,
    ) -> Result<ControlFlow, SpannedRuntimeError> {
        macro_rules! r {
            ($val:expr) => {
                match $val {
                    ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    ControlFlow::None(v) => v,
                }
            };
        }

        match &self.statement {
            _Statement::Let(ident, val) => {
                let val = r!(val.evaluate(program, runtime, scope)?);
                val.val_clone(runtime)?;
                let ty = val.ty();
                let id = runtime.memory.insert(val);
                scope.insert(ident, id, ty);

                Ok(ControlFlow::None(Value::Null))
            }

            _Statement::Assign(trg, val) => {
                // unwrap the target to find a reference
                let trg = match &trg.expression {
                    _Expression::Dereferece(e) => r!(e.evaluate(program, runtime, scope)?),
                    _ => return Err(SpannedRuntimeError::with_span(InvalidAssignTarget, self.span))
                };

                // make sure the expression gave you a reference
                if let Value::Ref(id, ty) = &trg {
                    let val = r!(val.evaluate(program, runtime, scope)?);

                    // insure trg and val have the same type
                    if val.ty() == *ty {
                        val.val_clone(runtime)?;

                        runtime.memory.get(id)?.clone().drop(runtime)?; // drop the previous value, in targets place
                        
                        *runtime.memory.get_mut(id)? = val; // replace target with val

                        Ok(ControlFlow::None(Value::Null))
                    } else {
                        Err(SpannedRuntimeError::with_span(
                            InequalLeftRightHandTypes,
                            self.span,
                        ))
                    }
                } else {
                    Err(SpannedRuntimeError::with_span(
                        InvalidAssignTarget,
                        self.span,
                    ))
                }
            }

            _Statement::Expression(expr) => {
                r!(expr.evaluate(program, runtime, scope)?);
                Ok(ControlFlow::None(Value::Null))
            }

            _Statement::Block(block) => block.evaluate(program, runtime, scope),
        }
    }
}

#[derive(Debug)]
pub enum _Expression {
    Literal(Value),
    Variable(String),
    Dereferece(Box<Expression>),
    Reference(Box<Expression>),
    Block(Block),
}

#[derive(Debug)]
pub struct Expression {
    pub expression: _Expression,
    pub span: Span,
}

impl Expression {
    pub fn evaluate(
        &self,
        program: &Program,
        runtime: &mut Runtime,
        scope: &mut Scope,
    ) -> Result<ControlFlow, SpannedRuntimeError> {
        macro_rules! r {
            ($val:expr) => {
                match $val {
                    ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    ControlFlow::None(val) => val,
                }
            };
        }

        match &self.expression {
            _Expression::Literal(val) => Ok(ControlFlow::None(val.clone())),

            _Expression::Variable(var) => {
                if let Some((id, ty)) = scope.get(var) {
                    Ok(ControlFlow::None(Value::Ref(id.clone(), ty.clone())))
                } else {
                    Err(SpannedRuntimeError::with_span(UndefinedVariable, self.span))
                }
            }

            _Expression::Dereferece(reference) => {
                let reference = r!(reference.evaluate(program, runtime, scope)?);

                if let Value::Ref(id, _) = &reference {
                    let val = runtime.memory.get(id)?.clone();

                    Ok(ControlFlow::None(val))
                } else {
                    Err(SpannedRuntimeError::with_span(
                        InvalidDerefTarget,
                        self.span,
                    ))
                }
            }

            _Expression::Reference(target) => {
                let target = match &target.expression {
                    _Expression::Dereferece(e) => r!(e.evaluate(program, runtime, scope)?),
                    _ => r!(target.evaluate(program, runtime, scope)?),
                };

                if let Value::Ref(id, ty) = &target {
                    Ok(ControlFlow::None(Value::Ref(*id, ty.clone())))
                } else {
                    Err(SpannedRuntimeError::with_span(
                        InvalidReferenceTarget,
                        self.span,
                    ))
                }
            }

            _Expression::Block(block) => {
                for statement in &block.statements {
                    r!(statement.evaluate(program, runtime, scope)?);
                }

                if let Some(expression) = &block.expression {
                    expression.evaluate(program, runtime, scope)
                } else {
                    Ok(ControlFlow::None(Value::Null))
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

        assert_eq!(
            Value::String("foo is a bar".to_string()),
            parser.parse(r#""foo is a bar""#).unwrap()
        );
        assert_eq!(Value::I32(42), parser.parse("42").unwrap());
        assert_eq!(Value::F32(420.69), parser.parse("420.69").unwrap());
    }

    #[test]
    fn expression_parsing() {
        let parser = grammar::ExprParser::new();
    }
}
