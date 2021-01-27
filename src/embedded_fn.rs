use crate::fn_storage::*;
use crate::function::*;
use crate::variant::*;
use std::sync::Arc;

/// Used to denote which type of function the [`IntoEmbeddedFn`] should turn into.
pub struct EmbFn;

#[derive(Clone)]
pub struct EmbeddedFn {
    runner: Arc<dyn Fn(Vec<Variable>) -> Result<UnionCell, ()>>,
}

impl std::fmt::Debug for EmbeddedFn {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "EmbeddedFn")?;

        Ok(())
    }
}

impl EmbeddedFn {
    pub fn run(&self, input: Vec<Variable>) -> Result<UnionCell, ()> {
        (self.runner)(input)
    }
}

macro_rules! def_register {
    ($($ident:ident),*) => {
        impl<T, $($ident,)* R, F> IntoEmbeddedFn<T, ($(&$ident,)*), R, EmbFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident> + 'static,)*
            R: Variant,
            F: Fn($($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn into_embedded_fn(self) -> FnType<T> {
                FnType::EmbeddedFn(
                    EmbeddedFn {
                        runner: Arc::new(move |mut _input| {
                            let mut _iter = _input.into_iter();

                            $(
                                #[allow(non_snake_case)]
                                let $ident = $ident::map(&mut _iter.next().unwrap()).ok_or(())?;
                            )*

                            Ok(UnionCell::new(self($($ident),*)))
                        }),
                    }
                )
            }
        }

        impl<$($ident,)* R, F> IntoFnParameters<($(&$ident,)*), R, EmbFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident> + 'static,)*
            R: Variant,
            F: Fn($($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn fn_parameters() -> Vec<FnParameter> {
                vec![$(FnParameter::from_embedded_fn_parameter::<$ident>(),)*]
            }
        }

        impl<T, M, $($ident,)* R, F> IntoEmbeddedFn<T, (&mut M, $(&$ident,)*), R, EmbFn> for F
        where
            $($ident: Variant + Clone,)*
            M: Variant,
            R: Variant,
            F: Fn(&mut M, $($ident,)*) -> R + 'static,
        {
            #[inline(always)]
            fn into_embedded_fn(self) -> FnType<T> {
                FnType::EmbeddedFn(
                    EmbeddedFn {
                        runner: Arc::new(move |mut _input| {
                            let mut _iter = _input.into_iter();

                            let mut var = match _iter.next().unwrap().cloned() {
                                Union::Reference(var) => var,
                                _ => return Err(()),
                            };

                            var.map_mut(|a| {
                                let a: &mut M = a.downcast_mut().expect("Should be unreachable");

                                $(
                                    #[allow(non_snake_case)]
                                    let $ident = by_value(_iter.next().unwrap().cloned());
                                )*

                                Ok(UnionCell::new(self(a, $($ident),*)))
                            })
                        }),
                    }
                )
            }
        }

        impl<M, $($ident,)* R, F> IntoFnParameters<(&mut M, $(&$ident,)*), R, EmbFn> for F
        where
            $($ident: EmbeddedFnParameter<$ident> + 'static,)*
            M: Variant,
            R: Variant,
            F: Fn(&mut M, $($ident,)*) -> R + 'static,
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
