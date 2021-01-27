use crate::error::*;
use crate::function::*;
use crate::variant::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FnParameter {
    Specified(UnionType),
    Unspecified,
}

impl FnParameter {
    pub fn from(ty: Option<UnionType>) -> Self {
        match ty {
            Some(ty) => Self::Specified(ty),
            None => Self::Unspecified,
        }
    }

    pub fn from_embedded_fn_parameter<T>() -> Self
    where
        T: EmbeddedFnParameter<T> + Any,
    {
        if TypeId::of::<T>() == TypeId::of::<Union>() {
            FnParameter::Unspecified
        } else {
            FnParameter::Specified(T::union_type())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnSignature {
    pub ident: String,
    pub params: Vec<FnParameter>,
}

impl FnSignature {
    pub fn from<F, P, R, U>(ident: impl Into<String>, f: &F) -> Self
    where
        F: IntoFnParameters<P, R, U>,
    {
        Self {
            ident: ident.into(),
            params: f.into_fn_parameters(),
        }
    }
}

pub trait IntoFnParameters<P, R, U> {
    fn fn_parameters() -> Vec<FnParameter>;

    fn into_fn_parameters(&self) -> Vec<FnParameter> {
        Self::fn_parameters()
    }
}

#[derive(Debug)]
struct FnStorageBranch<T> {
    branches: HashMap<FnParameter, FnStorageBranch<T>>,
    end: Option<FnType<T>>,
}

impl<T> FnStorageBranch<T> {
    pub fn new() -> Self {
        Self {
            branches: HashMap::new(),
            end: None,
        }
    }

    pub fn merge_override(&mut self, branch: FnStorageBranch<T>) {
        for (parameter, branch) in branch.branches {
            match self.branches.get_mut(&parameter) {
                Some(b) => b.merge_override(branch),
                None => {
                    self.branches.insert(parameter, branch);
                }
            }
        }

        self.end = branch.end;
    }

    pub fn register_fn<I: Iterator<Item = FnParameter>>(
        &mut self,
        mut iter: I,
        fn_type: FnType<T>,
    ) -> Result<(), ErrorKind> {
        match iter.next() {
            Some(p) => self
                .branches
                .entry(p)
                .or_insert(FnStorageBranch::new())
                .register_fn(iter, fn_type),
            None => match self.end {
                Some(_) => Err(ErrorKind::FunctionRedefinition),
                None => {
                    self.end = Some(fn_type);
                    Ok(())
                }
            },
        }
    }

    pub fn get_fn<'a, I: Iterator<Item = &'a FnParameter>>(
        &self,
        mut iter: I,
    ) -> Result<&FnType<T>, ErrorKind> {
        match iter.next() {
            Some(p) => match self.branches.get(&p) {
                Some(b) => b.get_fn(iter),
                None => match self.branches.get(&FnParameter::Unspecified) {
                    Some(b) => b.get_fn(iter),
                    None => Err(ErrorKind::UndefinedFunction),
                },
            },
            None => match &self.end {
                Some(fn_type) => Ok(fn_type),
                None => Err(ErrorKind::UndefinedFunction),
            },
        }
    }
}

impl<T> Clone for FnStorageBranch<T> {
    fn clone(&self) -> Self {
        Self {
            branches: self.branches.clone(),
            end: self.end.clone(),
        }
    }
}

#[derive(Debug)]
pub struct FnStorage<T> {
    functions: HashMap<String, FnStorageBranch<T>>,
}

impl<T> FnStorage<T> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn merge_override(&mut self, fn_storage: FnStorage<T>) {
        for (ident, branch) in fn_storage.functions {
            match self.functions.get_mut(&ident) {
                Some(b) => b.merge_override(branch),
                None => {
                    self.functions.insert(ident, branch);
                }
            }
        }
    }

    pub fn register_fn(
        &mut self,
        fn_signature: FnSignature,
        fn_type: FnType<T>,
    ) -> Result<(), ErrorKind> {
        self.functions
            .entry(fn_signature.ident)
            .or_insert(FnStorageBranch::new())
            .register_fn(fn_signature.params.into_iter(), fn_type)
    }

    pub fn get_fn(&self, fn_signature: &FnSignature) -> Result<&FnType<T>, ErrorKind> {
        match self.functions.get(&fn_signature.ident) {
            Some(b) => b.get_fn(fn_signature.params.iter()),
            None => Err(ErrorKind::UndefinedFunction),
        }
    }
}

impl<T> Clone for FnStorage<T> {
    fn clone(&self) -> Self {
        Self {
            functions: self.functions.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_embedded_fn_parameter() {
        assert_eq!(
            FnParameter::from_embedded_fn_parameter::<Mut<i32>>(),
            FnParameter::Specified(UnionType::Reference(Box::new(UnionType::Int))),
        );

        assert_eq!(
            FnParameter::from_embedded_fn_parameter::<Union>(),
            FnParameter::Unspecified,
        );

        assert_eq!(
            FnParameter::from_embedded_fn_parameter::<bool>(),
            FnParameter::Specified(UnionType::Bool),
        );
    }
}
