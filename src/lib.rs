use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod memory;
pub mod runtime;

lalrpop_mod!(pub grammar);
