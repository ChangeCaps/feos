use crate::ast::*;
use crate::control_flow::*;
use crate::error::*;
use crate::fn_storage::*;
use crate::runtime::*;
use crate::scope::*;
use crate::span::*;
use crate::to_fn_input::*;
use crate::variant::*;

impl<'a, T> Runtime<'a, T> {
    #[inline(always)]
    pub fn eval_expr(
        &mut self,
        expr: &Spanned<Expr>,
        scope: &mut Scope<T>,
    ) -> Result<Variable, ControlFlow> {
        match &**expr {
            Expr::Literal { variant } => Ok(Variable::unspecified(variant.clone())),

            Expr::Assign { target, variable } => {
                let mut target = self.eval_expr(target, scope)?;
                let variant = self.eval_expr(variable, scope)?;

                if target.type_specified {
                    if target.ty() != variant.ty() {
                        return Err(
                            Error::new(ErrorKind::TypeMismatch, &self.source, expr.span).into()
                        );
                    }
                }

                target.map_mut(|v| {
                    *v = variant.cloned();
                });

                Ok(().into())
            }

            Expr::NegationOp { expr } => {
                let variable = self.eval_expr(expr, scope)?;

                let params = vec![variable.clone()];

                let fn_signature = FnSignature {
                    ident: "!".into(),
                    params: params.to_fn_parameters(),
                };

                match scope.get_fn(&fn_signature) {
                    Ok(op_fn) => {
                        Ok(op_fn.run(&expr.span, self, scope, params.to_fn_input())?)
                    },
                    Err(_) => {
                        variable.map(|u| 
                            match u {
                                Union::Bool(b) => Ok(Variable::specified(Union::Bool(!b))),
                                _ => Err(Error::from_raw(ErrorKind::UndefinedFunction, "!").into())
                            }
                        )
                    }
                }
            }

            Expr::BinOp { lhs, rhs, op } => {
                let lhs = self.eval_expr(lhs, scope)?;
                let rhs = self.eval_expr(rhs, scope)?;

                let params = vec![lhs.clone(), rhs.clone()];

                let fn_signature = FnSignature {
                    ident: op.inner.clone(),
                    params: params.to_fn_parameters(),
                };

                match scope.get_fn(&fn_signature) {
                    Ok(op_fn) => {
                        Ok(op_fn.run(&op.span, self, scope, params.to_fn_input())?)
                    },
                    Err(_) => {
                        match crate::internal_binop::internal_binop(lhs.cloned(), rhs.cloned(), op) {
                            Some(v) => Ok(Variable::specified(v)),
                            None => Err(Error::new(ErrorKind::UndefinedFunction, &self.source, op.span).into()),
                        }
                    }
                }
            }

            Expr::Variable { ident } => {
                let variable = scope.get_variable_mut(ident);

                match variable {
                    Some(v) => Ok(v.get_shared()),
                    None => Err(
                        Error::new(ErrorKind::UndefinedVariable, &self.source, ident.span).into(),
                    ),
                }
            }

            Expr::Reference { expr } => {
                let mut variable = self.eval_expr(expr, scope)?;

                let referenced = Variable {
                    type_specified: variable.type_specified,
                    union: Union::Reference(Box::new(variable.get_shared())).into(),
                };

                Ok(referenced)
            }

            Expr::Dereference { expr } => {
                let variable = self.eval_expr(expr, scope)?;

                if let Union::Reference(mut referenced) = variable.cloned() {
                    Ok(referenced.get_shared())
                } else {
                    Err(Error::new(ErrorKind::InvalidDerefTarget, &self.source, expr.span).into())
                }
            }

            Expr::Block { block } => {
                let mut scope = scope.sub();

                self.eval_block(block, &mut scope)
            }

            Expr::If {
                check,
                block,
                else_block,
            } => {
                let check = self.eval_expr(check, scope)?.map(|v| match v.as_bool() {
                    Some(b) => Ok(b),
                    None => Err(Error::new(
                        ErrorKind::TypeMismatch,
                        &self.source,
                        check.span,
                    )),
                })?;

                if check {
                    self.eval_block(block, scope)
                } else {
                    if let Some(else_block) = else_block {
                        self.eval_expr(else_block, scope)
                    } else {
                        Ok(Variable::unspecified(Union::from(())))
                    }
                }
            }

            Expr::FnCall { ident, params } => {
                let params = {
                    let mut p = Vec::with_capacity(params.len());

                    for param in params {
                        p.push(self.eval_expr(param, scope)?);
                    }

                    p
                };

                Ok(self.call_fn(&**ident, params, scope)?)
            }

            Expr::MethodCall {
                ident,
                caller,
                params,
            } => {
                let caller = self.eval_expr(caller, scope)?;

                let mut p = Vec::with_capacity(params.len() + 1);

                p.push(caller);

                for param in params {
                    p.push(self.eval_expr(param, scope)?);
                }

                match self.call_fn(&**ident, p.clone(), scope) {
                    Ok(v) => Ok(v),
                    Err(Error {
                        kind: ErrorKind::UndefinedFunction,
                        ..
                    }) => {
                        p[0] = Variable::new(
                            Union::Reference(Box::new(p[0].get_shared())),
                            p[0].type_specified,
                        );

                        Ok(self.call_fn(&**ident, p, scope)?)
                    }
                    Err(err) => Err(err.into()),
                }
            }

            Expr::WhileLoop { expr, block } => {
                loop {
                    let check = self.eval_expr(expr, scope)?.map(|v| match v.as_bool() {
                        Some(b) => Ok(b),
                        None => Err(Error::new(ErrorKind::TypeMismatch, &self.source, expr.span)),
                    })?;

                    if check {
                        self.eval_block(block, scope)?;
                    } else {
                        break;
                    }
                }

                Ok(Variable::specified(Union::Unit(())))
            }

            Expr::ForLoop { ident, expr, block } => {
                let iterator = self.eval_expr(expr, scope)?;

                // try to turn into iter
                let mut iterator = match self.call_fn("into_iter", vec![iterator.clone()], scope) {
                    Ok(iterator) => iterator,
                    Err(Error {
                        kind: ErrorKind::UndefinedFunction,
                        ..
                    }) => iterator,
                    Err(err) => return Err(err.into()),
                };

                let params = vec![Variable::specified(Union::Reference(Box::new(
                    iterator.get_shared(),
                )))];

                let fn_signature = FnSignature {
                    ident: "iter_next".into(),
                    params: params.to_fn_parameters(),
                };

                let iter_next = scope
                    .get_fn(&fn_signature)
                    .map_err(|err| {
                        Error::from_raw(err, format!("{} is not an iterator", ident.inner))
                    })?
                    .clone();

                loop {
                    let variable =
                        iter_next.run(&expr.span, self, scope, params.clone().to_fn_input())?;

                    let option = variable
                        .cloned()
                        .downcast_ref::<Option<Union>>()
                        .ok_or(Error::new(ErrorKind::TypeMismatch, &self.source, expr.span))?
                        .clone();

                    match option {
                        Some(union) => {
                            let mut scope = scope.sub();

                            scope.register_variable(&**ident, Variable::unspecified(union));

                            self.eval_block(block, &mut scope)?;
                        }
                        None => break,
                    }
                }

                Ok(Variable::unspecified(Union::Unit(())))
            }
        }
    }
}
