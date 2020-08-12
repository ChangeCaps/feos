use crate::ast::{RuntimeError::*, Span, SpannedRuntimeError};
use std::collections::HashMap;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct MemoryID(u32);

impl MemoryID {
    pub fn next(&self) -> Self {
        MemoryID(self.0 + 1)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
    Ref(MemoryID, ValueType),
    TraitObject(MemoryID, Vec<String>),
    Null,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueType {
    I32,
    F32,
    Bool,
    String,
    Ref(Box<ValueType>),
    TraitObject(Vec<String>),
    Null,
}

impl Value {
    pub fn ty(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::F32(_) => ValueType::F32,
            Value::Bool(_) => ValueType::Bool,
            Value::String(_) => ValueType::String,
            Value::Ref(_, ty) => ValueType::Ref(Box::new(ty.clone())),
            Value::TraitObject(_, traits) => ValueType::TraitObject(traits.clone()),
            Value::Null => ValueType::Null,
        }
    }

    pub fn val_clone(
        &self,
        runtime: &mut crate::runtime::Runtime,
    ) -> Result<(), SpannedRuntimeError> {
        match self {
            Value::Ref(id, _) => {
                runtime.memory.add_reference(id)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn equal(&self, rhs: &Self) -> bool {
        match self {
            _ => self == rhs,
        }
    }

    pub fn drop(&self, runtime: &mut crate::runtime::Runtime) -> Result<(), SpannedRuntimeError> {
        match self {
            Value::Ref(id, _) => {
                runtime.memory.remove(id)?;
            }
            _ => (),
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MemoryEntry {
    value: Value,
    references: u32,
}

impl MemoryEntry {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            references: 0,
        }
    }

    pub fn add_reference(&mut self) {
        self.references += 1;
    }

    pub fn remove_reference(&mut self) -> bool {
        if self.references == 0 {
            true
        } else {
            self.references -= 1;
            false
        }
    }

    pub fn reference_count(&self) -> u32 {
        self.references + 1
    }
}

#[derive(Clone, Debug)]
pub struct Memory {
    memory: HashMap<MemoryID, MemoryEntry>,
    next_id: MemoryID,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            next_id: MemoryID(0),
        }
    }

    pub fn insert(&mut self, value: Value) -> MemoryID {
        let entry = MemoryEntry::new(value);

        self.memory.insert(self.next_id, entry);

        let id = self.next_id;
        self.next_id = self.next_id.next();

        id
    }

    pub fn remove(&mut self, id: &MemoryID) -> Result<(), SpannedRuntimeError> {
        if let Some(entry) = self.memory.get_mut(id) {
            if entry.remove_reference() {
                self.memory.remove(id);
            }

            Ok(())
        } else {
            Err(SpannedRuntimeError::new(InvalidMemoryID))
        }
    }

    pub fn add_reference(&mut self, id: &MemoryID) -> Result<(), SpannedRuntimeError> {
        if let Some(entry) = self.memory.get_mut(id) {
            entry.add_reference();

            Ok(())
        } else {
            Err(SpannedRuntimeError::new(InvalidMemoryID))
        }
    }

    pub fn get(&self, id: &MemoryID) -> Result<&Value, SpannedRuntimeError> {
        if let Some(entry) = self.memory.get(id) {
            Ok(&entry.value)
        } else {
            Err(SpannedRuntimeError::new(InvalidMemoryID))
        }
    }

    pub fn get_mut(&mut self, id: &MemoryID) -> Result<&mut Value, SpannedRuntimeError> {
        if let Some(entry) = self.memory.get_mut(id) {
            Ok(&mut entry.value)
        } else {
            Err(SpannedRuntimeError::new(InvalidMemoryID))
        }
    }
}
