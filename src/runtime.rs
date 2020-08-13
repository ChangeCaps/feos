use crate::ast::*;
use crate::memory::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Scope {
    variables: HashMap<String, (MemoryID, ValueType)>,
    added: Vec<MemoryID>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            added: Vec::new(),
        }
    }

    pub fn sub(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            added: Vec::new(),
        }
    }

    pub fn get(&self, var: &String) -> Option<&(MemoryID, ValueType)> {
        self.variables.get(var)
    }

    pub fn add(&mut self, id: MemoryID) {
        self.added.push(id);
    }

    pub fn insert(&mut self, var: &String, id: MemoryID, ty: ValueType) {
        self.variables.insert(var.clone(), (id, ty));
        self.added.push(id);
    }

    pub fn drop(self, runtime: &mut Runtime) -> Result<(), SpannedRuntimeError> {
        for id in self.added {
            if let Some(val) = runtime.memory.remove(&id)? {
                val.drop(runtime)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Runtime {
    pub(crate) memory: Memory,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }
}
