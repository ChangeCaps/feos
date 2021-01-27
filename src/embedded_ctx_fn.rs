use crate::function::*;
use crate::fn_storage::*;
use crate::variant::*;
use std::sync::Arc;

/// Used to denote which type of function the [`IntoEmbeddedFn`] should turn into.
pub struct EmbCtxFn;

pub struct EmbeddedCtxFn<T> {
    runner: Arc<dyn Fn(&mut T, Vec<Variable>) -> Result<UnionCell, ()>>,
}

impl<T> std::fmt::Debug for EmbeddedCtxFn<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "EmbeddedCtxFn")?;

        Ok(())
    }
}

impl<T> Clone for EmbeddedCtxFn<T> {
    fn clone(&self) -> Self {
        Self {
            runner: self.runner.clone(),
        }
    }
}

impl<T> EmbeddedCtxFn<T> {
    pub fn run(&self, ctx: &mut T, input: Vec<Variable>) -> Result<UnionCell, ()> {
        (self.runner)(ctx, input)
    }
}

macro_rules! def_register {
    ($($ident:ident),*) => {
        impl<T, $($ident,)* R, F> IntoEmbeddedFn<T, ($(&$ident,)*), R, EmbCtxFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident>,)*
            R: Variant,
            F: Fn(&mut T, $($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn into_embedded_fn(self) -> FnType<T> {
                FnType::EmbeddedCtxFn(
                    EmbeddedCtxFn {
                        runner: Arc::new(move |ctx, mut _input| {
                            let mut _iter = _input.into_iter();

                            $(
                                #[allow(non_snake_case)]
                                let $ident = $ident::map(&mut _iter.next().unwrap()).ok_or(())?;
                            )*

                            Ok(UnionCell::new(self(ctx, $($ident,)*)))
                        }),
                    }
                )
            }
        }

        impl<T, $($ident,)* R, F> IntoFnParameters<(&mut T, $(&$ident,)*), R, EmbCtxFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident> + 'static,)*
            R: Variant,
            F: Fn(&mut T, $($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn fn_parameters() -> Vec<FnParameter> {
                vec![$(FnParameter::from_embedded_fn_parameter::<$ident>(),)*]
            }
        }

        impl<T, M, $($ident,)* R, F> IntoEmbeddedFn<T, (&mut M, $(&$ident,)*), R, EmbCtxFn> for F
        where
            M: Variant,
            $($ident: Variant + Clone,)*
            R: Variant,
            F: Fn(&mut T, &mut M, $($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn into_embedded_fn(self) -> FnType<T> {
                FnType::EmbeddedCtxFn(
                    EmbeddedCtxFn {
                        runner: Arc::new(move |ctx, mut _input| {
                            let mut _iter = _input.into_iter();

                            let mut var = match _iter.next().unwrap().cloned() {
                                Union::Reference(var) => var,
                                _ => return Err(()),
                            };

                            var.map_mut(|v| {
                                let m = v.downcast_mut::<M>().ok_or(())?;

                                $(
                                    #[allow(non_snake_case)]
                                    let $ident = by_value(_iter.next().unwrap().cloned());
                                )*

                                Ok(UnionCell::new(self(ctx, m, $($ident,)*)))
                            })
                        }),
                    }
                )
            }
        }

        impl<T, M, $($ident,)* R, F> IntoFnParameters<(&mut T, &mut M, $(&$ident,)*), R, EmbCtxFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident> + 'static,)*
            M: Variant,
            R: Variant,
            F: Fn(&mut T, &mut M, $($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn fn_parameters() -> Vec<FnParameter> {
                vec![
                    FnParameter::Specified(UnionType::Reference(Box::new(UnionType::from::<M>()))),
                    $(FnParameter::from_embedded_fn_parameter::<$ident>(),)*
                ]
            }
        }
    };
}

def_register!();
def_register!(A);
def_register!(A, B);
def_register!(A, B, C);
def_register!(A, B, C, D);
def_register!(A, B, C, D, E);
def_register!(A, B, C, D, E, G);
def_register!(A, B, C, D, E, G, H);
def_register!(A, B, C, D, E, G, H, I);
def_register!(A, B, C, D, E, G, H, I, K);
def_register!(A, B, C, D, E, G, H, I, K, J);
def_register!(A, B, C, D, E, G, H, I, K, J, L);
def_register!(A, B, C, D, E, G, H, I, K, J, L, N);
