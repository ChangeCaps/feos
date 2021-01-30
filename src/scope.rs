use crate::error::ErrorKind;
use crate::fn_storage::*;
use crate::function::*;
use crate::module::*;
use crate::variant::*;

pub struct Scope<T> {
    module: Module<T>,
    values: Vec<Variable>,
    idents: Vec<String>,
    start: usize,
    subs: Vec<(usize, Option<usize>)>,
}

impl<T> Clone for Scope<T> {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            values: self.values.clone(),
            idents: self.idents.clone(),
            start: self.start.clone(),
            subs: self.subs.clone(),
        }
    }
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Self {
            module: Module::new(),
            values: Vec::with_capacity(64),
            idents: Vec::with_capacity(64),
            start: 0,
            subs: Vec::with_capacity(16),
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

    #[inline(always)]
    pub fn get_fn(&self, signature: &FnSignature) -> Result<&FnType<T>, ErrorKind> {
        self.module.get_fn(signature)
    }

    pub fn push(&mut self, ident: impl Into<String>, value: impl Into<Variable>) {
        self.idents.push(ident.into());
        self.values.push(value.into());
    }

    pub fn get_index(&self, ident: &String) -> Option<usize> {
        self.idents[self.start..]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, name)| {
                if name == ident {
                    Some(self.start + i)
                } else {
                    None
                }
            })
    }

    pub fn get_variable(&self, ident: &String) -> Option<&Variable> {
        match self.get_index(ident) {
            Some(index) => Some(&self.values[index]),
            None => None,
        }
    }

    pub fn get_variable_mut(&mut self, ident: &String) -> Option<&mut Variable> {
        match self.get_index(ident) {
            Some(index) => Some(&mut self.values[index]),
            None => None,
        }
    }

    #[inline(always)]
    pub fn sub(&mut self, set_start: bool) {
        let start = if set_start { Some(self.start) } else { None };

        if set_start {
            self.start = self.values.len();
        }

        self.subs.push((self.values.len(), start));
    }

    #[inline(always)]
    pub fn rev_sub(&mut self) {
        let (len, start) = self.subs.pop().expect("failed to reverse a sub scope");
        self.values.truncate(len);
        self.idents.truncate(len);

        if let Some(start) = start {
            self.start = start;
        }
    }
}
