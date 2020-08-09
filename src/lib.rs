use lalrpop_util::lalrpop_mod;

mod memory;
pub mod runtime;
mod ast;

lalrpop_mod!(pub grammar);