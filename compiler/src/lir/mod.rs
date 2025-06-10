pub mod compile;
pub mod parse;

pub use std::collections::HashMap;

pub type Name = String;

pub type Symbol = u32;

pub type Arity = u16;

pub type Index = u64;

pub type Program = HashMap<String, Global>;

#[derive(Debug, PartialEq)]
pub enum Global {
    Const { arity: Arity, symbol: Symbol },
    Fun { arity: Arity, block: Block },
}

pub type Block = Vec<Op>;

#[derive(Debug, PartialEq)]
pub enum Op {
    LoadGlobal {
        global: Name,
    },
    LoadArg {
        name: Name,
        var: Name,
        index: Index,
    },
    NewApp {
        name: Name,
        var: Name,
        args: Vec<Name>,
    },
    NewPartial {
        name: Name,
        var: Name,
        args: Vec<Name>,
    },
    ApplyPartial {
        name: Name,
        var: Name,
        args: Vec<Name>,
    },
    Copy {
        name: Name,
        var: Name,
    },
    Eval {
        var: Name,
    },
    FreeArgs {
        var: Name,
    },
    FreeTerm {
        var: Name,
    },
    ReturnSymbol {
        var: Name,
    },
    Return {
        var: Name,
    },
    Switch {
        var: Name,
        cases: Vec<Case>,
    },
    Todo,
}

#[derive(Debug, PartialEq)]
pub struct Case {
    pub global: Name,
    pub block: Block,
}
