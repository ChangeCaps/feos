pub mod ast;
pub mod control_flow;
pub mod embedded_ctx_fn;
pub mod embedded_fn;
pub mod engine;
pub mod error;
mod eval_expr;
mod eval_stmt;
pub mod fn_storage;
pub mod function;
pub mod module;
pub mod runtime;
pub mod scope;
pub mod span;
pub mod variant;
#[macro_use]
pub mod macros;
pub mod iron_std;
pub mod to_fn_input;
mod internal_binop;

pub use macros::*;

use lalrpop_util::*;

lalrpop_mod!(pub grammar);

#[macro_use]
pub mod prelude {
    pub use crate::{def_module, module_items};

    pub use crate::engine::*;
    pub use crate::runtime::*;
    pub use crate::variant::*;
}
