use crate::error::*;
use crate::fn_storage::*;
use crate::function::*;
use crate::runtime::*;
use crate::scope::*;
use crate::variant::*;
use std::path::PathBuf;

pub struct Engine<T> {
    scope: Scope<T>,
}

impl<T> Engine<T> {
    pub fn new() -> Self {
        let mut scope = Scope::new();

        let std = crate::iron_std::iron_std();
        let global = crate::iron_std::global_full();

        scope.register_module("std", std);
        scope.merge_module(global);

        Self { scope }
    }

    pub fn register_fn<P, R, F, U>(&mut self, ident: impl Into<String>, f: F) -> &mut Self
    where
        F: IntoEmbeddedFn<T, P, R, U>,
        F: IntoFnParameters<P, R, U>,
    {
        self.scope
            .register_fn(FnSignature::from(ident, &f), f.into_embedded_fn())
            .unwrap();

        self
    }

    pub fn eval_file(&self, ctx: &mut T, path: impl Into<PathBuf>) -> Result<Union, Error> {
        let source = std::fs::read_to_string(path.into()).unwrap();

        let program = crate::grammar::BlockParser::new().parse(&source).unwrap();

        let mut runtime = Runtime::new(ctx, source);
        let mut scope = self.scope.clone();

        runtime.run(&program, &mut scope)
    }
}
