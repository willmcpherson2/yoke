pub mod compile;
pub mod parse;

pub type Name = String;

pub type Symbol = u32;

pub type Arity = u16;

#[derive(Debug)]
pub struct Program {
    pub globals: Vec<Global>,
    pub main: Block,
}

#[derive(Debug)]
pub enum Global {
    Const {
        name: Name,
        arity: Arity,
        symbol: Symbol,
    },
    Fun {
        name: Name,
        symbol: Symbol,
        arity: Arity,
        block: Block,
    },
}

#[derive(Debug)]
pub struct Block(pub Vec<Op>);

#[derive(Debug)]
pub enum Op {
    LoadGlobal {
        name: Name,
        global: Name,
    },
    LoadArg {
        name: Name,
        var: Name,
        index: u64,
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
    Return {
        var: Name,
    },
    ReturnSymbol {
        var: Name,
    },
    Switch {
        var: Name,
        cases: Vec<Case>,
    },
    Abort,
}

#[derive(Debug)]
pub struct Case {
    pub symbol: Symbol,
    pub block: Block,
}
