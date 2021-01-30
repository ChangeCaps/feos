use crate::ast::*;
use crate::control_flow::*;
use crate::error::*;
use crate::function::*;
use crate::runtime::*;
use crate::scope::*;
use crate::span::*;

impl<'a, T> Runtime<'a, T> {
    #[inline(always)]
    pub fn eval_stmt(
        &mut self,
        stmt: &Spanned<Stmt>,
        scope: &mut Scope<T>,
    ) -> Result<(), ControlFlow> {
        match &**stmt {
            Stmt::Let { ident, ty, expr } => {
                let variable = self.eval_expr(expr, scope)?;

                if let Some(ty) = ty {
                    if *ty != variable.ty() {
                        return Err(
                            Error::new(ErrorKind::TypeMismatch, &self.source, stmt.span).into()
                        );
                    }
                }

                scope.push(ident.clone(), variable);

                Ok(())
            }

            Stmt::Expr { expr } => {
                self.eval_expr(expr, scope)?;

                Ok(())
            }

            Stmt::FnDef {
                fn_signature,
                block,
                parameter_idents,
                return_type,
            } => {
                let fn_type = FnType::<T>::Native {
                    block: block.clone(),
                    parameter_idents: parameter_idents.clone(),
                    return_type: return_type.clone(),
                };

                scope
                    .register_fn(fn_signature.clone(), fn_type)
                    .map_err(|err| Error::new(err, &self.source, stmt.span))?;

                Ok(())
            }
        }
    }
}
