use crate::error::ErrorKind;
use crate::fn_storage::*;
use crate::function::*;
use crate::module::*;
use crate::variant::*;
use std::collections::HashMap;

pub struct Scope<T> {
    module: Module<T>,
    variables: HashMap<String, Variable>,
}

impl<T> Clone for Scope<T> {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            variables: self.variables.clone(),
        }
    }
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Self {
            module: Module::new(),
            variables: HashMap::new(),
        }
    }

    pub fn merge_module(&mut self, module: Module<T>) {
        self.module.merge_module(module);
    }

    pub fn register_module(
        &mut self,
        ident: impl Into<String>,
        module: Module<T>,
    ) -> Option<Module<T>> {
        self.module.register_sub_module(ident, module)
    }

    pub fn register_fn(
        &mut self,
        signature: FnSignature,
        fn_type: FnType<T>,
    ) -> Result<(), ErrorKind> {
        self.module.register_fn_raw(signature, fn_type)
    }

    pub fn get_fn(&self, signature: &FnSignature) -> Result<&FnType<T>, ErrorKind> {
        self.module.get_fn(signature)
    }

    pub fn register_variable(&mut self, ident: impl Into<String>, variable: Variable) {
        self.variables.insert(ident.into(), variable);
    }

    pub fn get_variable(&self, ident: &String) -> Option<&Variable> {
        self.variables.get(ident)
    }

    pub fn get_variable_mut(&mut self, ident: &String) -> Option<&mut Variable> {
        self.variables.get_mut(ident)
    }

    pub fn sub_no_vars(&self) -> Self {
        Self {
            module: self.module.clone(),
            variables: HashMap::new(),
        }
    }

    pub fn sub(&mut self) -> Self {
        let variables = self
            .variables
            .iter_mut()
            .map(|(i, v)| (i.clone(), v.get_shared()))
            .collect();

        Scope {
            variables,
            module: self.module.clone(),
        }
    }
}
