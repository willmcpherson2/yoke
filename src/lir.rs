use inkwell::{
    context::Context,
    module::{Linkage, Module},
};
use std::path::Path;

pub type Name = u64;

pub type Symbol = u32;

pub type Arity = u16;

pub struct Prog {
    pub globals: Vec<Global>,
    pub funs: Vec<Fun>,
    pub main: Block,
}

pub struct Global {
    pub name: Name,
    pub symbol: Symbol,
    pub arity: Arity,
}

pub struct Fun {
    pub name: Name,
    pub arg_name: Name,
    pub symbol: Symbol,
    pub arity: Arity,
    pub block: Block,
}

pub type Block = Vec<Op>;

pub enum Op {
    LoadGlobal(LoadGlobal),
    LoadArg(LoadArg),
    NewApp(NewApp),
    NewPartial(NewPartial),
    ApplyPartial(ApplyPartial),
    Copy(Copy),
    FreeArgs(FreeArgs),
    FreeTerm(FreeTerm),
    Eval(Eval),
    Return(Return),
    ReturnSymbol(ReturnSymbol),
    Switch(Switch),
}

pub struct LoadGlobal {
    pub name: Name,
    pub symbol: Symbol,
}

pub struct LoadArg {
    pub name: Name,
    pub var: Name,
    pub index: usize,
}

pub struct NewApp {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

pub struct NewPartial {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

pub struct ApplyPartial {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

pub struct Copy {
    pub name: Name,
    pub var: Name,
}

pub struct FreeArgs {
    pub var: Name,
}

pub struct FreeTerm {
    pub var: Name,
}

pub struct Eval {
    pub name: Name,
    pub var: Name,
}

pub struct Return {
    pub var: Name,
}

pub struct ReturnSymbol {
    pub var: Name,
}

pub struct Switch {
    pub var: Name,
    pub cases: Vec<Case>,
}

pub struct Case {
    pub symbol: Symbol,
    pub block: Block,
}

pub fn compile(prog: Prog) {
    let context = Context::create();
    let builder = context.create_builder();

    let path = Path::new("target/rts.bc");
    let rts = Module::parse_bitcode_from_path(&path, &context).unwrap();

    rts.get_function("noop")
        .unwrap()
        .set_linkage(Linkage::Internal);

    prog.globals.into_iter().for_each(compile_global);
    prog.funs.into_iter().for_each(compile_fun);
    compile_main(prog.main);
}

fn compile_global(global: Global) {
    todo!()
}

fn compile_fun(fun: Fun) {
    todo!()
}

fn compile_main(entry: Vec<Op>) {
    todo!()
}
