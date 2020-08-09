use crate::ast::*;
use crate::memory::*;
use std::collections::HashMap;

pub struct Scope {
    variables: HashMap<String, (MemoryID, ValueType)>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, var: &String) -> Option<&(MemoryID, ValueType)> {
        self.variables.get(var)
    }

    pub fn insert(&mut self, var: &String, id: MemoryID, ty: ValueType) {
        self.variables.insert(var.clone(), (id, ty));
    }
}

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