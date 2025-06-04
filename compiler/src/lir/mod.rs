pub mod compile;
pub mod parse;

pub type Name = String;

pub type Symbol = u32;

pub type Arity = u16;

pub type Index = u64;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub globals: Vec<Global>,
    pub main: Block,
}

#[derive(Debug, PartialEq)]
pub enum Global {
    Const {
        name: Name,
        arity: Arity,
        symbol: Symbol,
    },
    Fun {
        name: Name,
        arity: Arity,
        block: Block,
    },
}

#[derive(Debug, PartialEq)]
pub struct Block(pub Vec<Op>);

#[derive(Debug, PartialEq)]
pub enum Op {
    LoadGlobal {
        name: Name,
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
        name: Name,
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
    Abort,
}

#[derive(Debug, PartialEq)]
pub struct Case {
    pub symbol: Symbol,
    pub block: Block,
}
