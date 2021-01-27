use crate::fn_storage::*;
use crate::span::*;
use crate::variant::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Spanned<Stmt>>,
    pub expr: Option<Spanned<Expr>>,
}

#[derive(Clone ,Debug)]
pub enum Stmt {
    Let {
        ident: String,
        ty: Option<UnionType>,
        expr: Spanned<Expr>,
    },

    Expr {
        expr: Spanned<Expr>,
    },

    FnDef {
        fn_signature: FnSignature,
        block: Arc<Spanned<Block>>,
        parameter_idents: Arc<Vec<String>>,
        return_type: FnParameter,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal {
        variant: Union,
    },

    Variable {
        ident: Spanned<String>,
    },

    Assign {
        target: Box<Spanned<Expr>>,
        variable: Box<Spanned<Expr>>,
    },

    NegationOp {
        expr: Box<Spanned<Expr>>,
    },

    BinOp {
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
        op: Spanned<String>,
    },

    Reference {
        expr: Box<Spanned<Expr>>,
    },

    Dereference {
        expr: Box<Spanned<Expr>>,
    },

    Block {
        block: Box<Block>,
    },

    If {
        check: Box<Spanned<Expr>>,
        block: Box<Spanned<Block>>,
        else_block: Option<Box<Spanned<Expr>>>,
    },

    FnCall {
        ident: Spanned<String>,
        params: Vec<Spanned<Expr>>,
    },

    MethodCall {
        ident: Spanned<String>,
        caller: Box<Spanned<Expr>>,
        params: Vec<Spanned<Expr>>,
    },

    WhileLoop {
        expr: Box<Spanned<Expr>>,
        block: Box<Spanned<Block>>,
    },

    ForLoop {
        ident: Spanned<String>,
        expr: Box<Spanned<Expr>>,
        block: Box<Spanned<Block>>,
    },
}
