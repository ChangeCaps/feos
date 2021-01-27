use crate::fn_storage::*;
use crate::function::*;
use crate::variant::*;
use std::any::TypeId;

pub trait ToFnInput {
    fn to_fn_input(self) -> Vec<Variable>;
    fn to_fn_parameters(&self) -> Vec<FnParameter>;
}

impl ToFnInput for Vec<Variable> {
    fn to_fn_input(self) -> Vec<Variable> {
        self
    }

    fn to_fn_parameters(&self) -> Vec<FnParameter> {
        self.iter()
            .map(|f| FnParameter::Specified(f.ty()))
            .collect()
    }
}

macro_rules! def_to_fn_input {
    ($($ident:ident: $field:tt),*) => {
        impl<$($ident),*> ToFnInput for ($($ident,)*)
        where
            $($ident: Variant + EmbeddedFnParameter<$ident>,)*
        {
            fn to_fn_input(self) -> Vec<Variable> {
                #[allow(unused_mut)]
                let mut input = Vec::new();

                $(
                    input.push(Variable::specified(Union::from(self.$field)));
                )*

                input
            }

            fn to_fn_parameters(&self) -> Vec<FnParameter> {
                #[allow(unused_mut)]
                let mut params = Vec::new();

                $(
                    if TypeId::of::<$ident>() == TypeId::of::<Union>() {
                        params.push(FnParameter::Unspecified);
                    } else {
                        params.push(FnParameter::Specified($ident::union_type()));
                    }
                )*

                params
            }
        }
    };
}

def_to_fn_input!(A: 0);
def_to_fn_input!(A: 0, B: 1);
def_to_fn_input!(A: 0, B: 1, C: 2);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, J: 8);
def_to_fn_input!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, J: 8, K: 9);
