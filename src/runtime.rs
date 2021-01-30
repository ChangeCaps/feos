use crate::ast::*;
use crate::control_flow::*;
use crate::error::*;
use crate::fn_storage::*;
use crate::scope::*;
use crate::span::*;
use crate::to_fn_input::*;
use crate::variant::*;

pub struct Runtime<'a, T> {
    pub ctx: &'a mut T,
    pub source: String,
}

impl<'a, T> Runtime<'a, T> {
    pub fn new(ctx: &'a mut T, source: String) -> Self {
        Self { ctx, source }
    }

    pub fn run(&mut self, program: &Block, scope: &mut Scope<T>) -> Result<Union, Error> {
        match self.eval_block(&program, scope) {
            Ok(variable) | Err(ControlFlow::Return(variable)) => Ok(variable.into_inner()),
            Err(ControlFlow::Error(error)) => Err(error),
        }
    }

    #[inline(always)]
    pub fn call_fn<I: ToFnInput>(
        &mut self,
        ident: impl Into<String>,
        input: I,
        scope: &mut Scope<T>,
    ) -> Result<Variable, Error> {
        let params = input.to_fn_parameters();
        let input = input.to_fn_input();

        let fn_signature = FnSignature {
            ident: ident.into(),
            params,
        };

        let fn_type = scope
            .get_fn(&fn_signature)
            .map_err(|err| {
                Error::from_raw(
                    err,
                    format!("{} {:?}", fn_signature.ident, fn_signature.params),
                )
            })?
            .clone();

        fn_type.run(&Span::new(0, 0), self, scope, input)
    }

    #[inline(always)]
    pub fn eval_block(
        &mut self,
        block: &Block,
        scope: &mut Scope<T>,
    ) -> Result<Variable, ControlFlow> {
        for stmt in &block.stmts {
            self.eval_stmt(stmt, scope)?;
        }

        match &block.expr {
            Some(expr) => self.eval_expr(expr, scope),
            None => Ok(Variable::unspecified(Union::from(()))),
        }
    }
}
