use crate::error::*;
use crate::fn_storage::*;
use crate::function::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Module<T> {
    sub_modules: HashMap<String, Module<T>>,
    functions: FnStorage<T>,
}

impl<T> Clone for Module<T> {
    fn clone(&self) -> Self {
        Self {
            sub_modules: self.sub_modules.clone(),
            functions: self.functions.clone(),
        }
    }
}

impl<T> Module<T> {
    pub fn new() -> Self {
        Self {
            sub_modules: HashMap::new(),
            functions: FnStorage::new(),
        }
    }

    /// Merges module into self, overriding on collisions.
    pub fn merge_module(&mut self, module: Module<T>) {
        for (ident, sub_module) in module.sub_modules {
            match self.sub_modules.get_mut(&ident) {
                Some(sub) => sub.merge_module(sub_module),
                None => {
                    self.sub_modules.insert(ident, sub_module);
                }
            }
        }

        self.functions.merge_override(module.functions);
    }

    pub fn register_sub_module(
        &mut self,
        ident: impl Into<String>,
        module: Module<T>,
    ) -> Option<Module<T>> {
        self.sub_modules.insert(ident.into(), module)
    }

    pub fn register_fn<P, R, F, U>(
        &mut self,
        ident: impl Into<String>,
        f: F,
    ) -> Result<(), ErrorKind>
    where
        F: IntoEmbeddedFn<T, P, R, U>,
        F: IntoFnParameters<P, R, U>,
    {
        let fn_type = f.into_embedded_fn();
        let ident = ident.into();
        let fn_parameters = F::fn_parameters();

        let fn_signature = FnSignature {
            ident,
            params: fn_parameters,
        };

        self.register_fn_raw(fn_signature, fn_type)
    }

    pub fn register_fn_raw(
        &mut self,
        fn_signature: FnSignature,
        fn_type: FnType<T>,
    ) -> Result<(), ErrorKind> {
        self.functions.register_fn(fn_signature, fn_type)
    }

    #[inline(always)]
    pub fn get_fn(&self, fn_signature: &FnSignature) -> Result<&FnType<T>, ErrorKind> {
        self.functions.get_fn(fn_signature)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::variant::*;

    #[test]
    fn register_fn() {
        fn a(_: &mut i32, _: Union) {}

        let mut module = Module::<()>::new();

        assert!(module.register_fn("test123", a).is_ok());
        assert!(module.register_fn("test123", a).is_err());
    }
}
