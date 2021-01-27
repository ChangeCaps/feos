use crate::ast::*;
use crate::control_flow::*;
use crate::embedded_ctx_fn::*;
use crate::embedded_fn::*;
use crate::error::*;
use crate::fn_storage::*;
use crate::runtime::*;
use crate::scope::*;
use crate::span::*;
use crate::variant::*;
use std::any::TypeId;
use std::sync::Arc;

pub(crate) fn by_value<T: Variant + Clone>(data: Union) -> T {
    if TypeId::of::<T>() == TypeId::of::<&str>() {
        let ref_str = data.as_str().unwrap();

        // SAFETY: we just checked and the types are identical, therefore
        // transmuting a reference to it must be safe
        let ref_t = unsafe { std::mem::transmute::<_, &T>(&ref_str) };
        ref_t.clone()
    } else {
        data.downcast_ref::<T>()
            .expect(&format!("cast: {}", std::any::type_name::<T>()))
            .clone()
    }
}

#[derive(Debug)]
pub enum FnType<T> {
    Native {
        block: Arc<Spanned<Block>>,
        parameter_idents: Arc<Vec<String>>,
        return_type: FnParameter,
    },
    EmbeddedFn(EmbeddedFn),
    EmbeddedCtxFn(EmbeddedCtxFn<T>),
}

impl<T> Clone for FnType<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Native {
                block,
                parameter_idents,
                return_type,
            } => Self::Native {
                block: block.clone(),
                parameter_idents: parameter_idents.clone(),
                return_type: return_type.clone(),
            },
            Self::EmbeddedFn(embedded_fn) => Self::EmbeddedFn(embedded_fn.clone()),
            Self::EmbeddedCtxFn(embedded_ctx_fn) => Self::EmbeddedCtxFn(embedded_ctx_fn.clone()),
        }
    }
}

impl<T> FnType<T> {
    pub fn from<P, R, F, U>(f: F) -> Self
    where
        F: IntoEmbeddedFn<T, P, R, U>,
    {
        f.into_embedded_fn()
    }

    pub fn run(
        &self,
        span: &Span,
        runtime: &mut Runtime<T>,
        scope: &Scope<T>,
        input: Vec<Variable>,
    ) -> Result<Variable, Error> {
        match self {
            Self::Native {
                block,
                parameter_idents,
                return_type,
            } => {
                let mut scope = scope.sub_no_vars();

                for (ident, union_cell) in parameter_idents.iter().zip(input.into_iter()) {
                    scope.register_variable(ident, union_cell);
                }

                let returned = match runtime.eval_block(block, &mut scope) {
                    Ok(v) => Ok(v),
                    Err(err) => match err {
                        ControlFlow::Return(v) => Ok(v),
                        ControlFlow::Error(err) => Err(err),
                    },
                }?;

                if let FnParameter::Specified(return_type) = return_type {
                    if *return_type == returned.ty() {
                        Ok(returned)
                    } else {
                        Err(Error::new(ErrorKind::TypeMismatch, &runtime.source, block.span).into())
                    }
                } else {
                    Ok(returned)
                }
            }
            Self::EmbeddedFn(embedded_fn) => match embedded_fn.run(input) {
                Ok(union) => Ok(Variable::specified(union)),
                Err(_) => Err(Error::new(ErrorKind::Unreachable, &runtime.source, *span).into()),
            },
            Self::EmbeddedCtxFn(embedded_ctx_fn) => match embedded_ctx_fn.run(runtime.ctx, input) {
                Ok(union) => Ok(Variable::specified(union)),
                Err(_) => Err(Error::new(ErrorKind::Unreachable, &runtime.source, *span).into()),
            },
        }
    }
}

pub trait IntoEmbeddedFn<T, P, R, U> {
    fn into_embedded_fn(self) -> FnType<T>;
}

pub trait EmbeddedFnParameter<T>: Sized {
    fn map(union: &mut UnionCell) -> Option<Self>;

    fn union_type() -> UnionType;
}

impl<T: Variant + Clone> EmbeddedFnParameter<T> for T {
    fn map(union: &mut UnionCell) -> Option<Self> {
        Some(by_value(union.cloned()))
    }

    fn union_type() -> UnionType {
        UnionType::from::<T>()
    }
}

impl<T: Variant> EmbeddedFnParameter<Mut<T>> for Mut<T> {
    fn map(union: &mut UnionCell) -> Option<Self> {
        match union.cloned() {
            Union::Reference(mut r) => Some(r.get_lock()),
            _ => None,
        }
    }

    fn union_type() -> UnionType {
        UnionType::Reference(Box::new(UnionType::from::<T>()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_embedded_function() {
        fn a(_: i32, _: Mut<f32>) {}

        FnType::<bool>::from(a);
        FnType::<()>::from(a);
        FnType::<String>::from(a);

        fn b(_: &mut i32, _: f32) {}

        FnType::<bool>::from(b);
        FnType::<()>::from(b);
        FnType::<String>::from(b);

        fn c(_: &mut bool, _: i32, _: Mut<f32>) {}

        FnType::<bool>::from(c);

        fn d(_: &mut bool, _: &mut i32, _: f32) {}

        FnType::<bool>::from(d);
    }

    #[test]
    fn into_fn_parameters() {}
}
