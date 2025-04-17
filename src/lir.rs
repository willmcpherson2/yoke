use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, StructType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};
use std::{collections::HashMap, path::Path};

pub type Name = &'static str;

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
    pub global: Name,
}

pub struct LoadArg {
    pub name: Name,
    pub var: Name,
    pub index: u64,
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

struct Unit<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    term_type: StructType<'ctx>,
    fun_type: FunctionType<'ctx>,
    locals: Vec<HashMap<&'ctx str, PointerValue<'ctx>>>,
}

impl<'ctx> Unit<'ctx> {
    fn add_scope(&mut self) {
        self.locals.push(HashMap::new())
    }

    fn define(&mut self, name: &'ctx str, value: PointerValue<'ctx>) {
        self.locals.last_mut().unwrap().insert(name, value);
    }

    fn lookup(&self, var: &str) -> PointerValue<'ctx> {
        for scope in self.locals.iter().rev() {
            if let Some(local) = scope.get(var) {
                return local.clone();
            }
        }
        panic!("no local with name: {}", var)
    }

    fn clear_locals(&mut self) {
        self.locals.clear()
    }

    fn clear_scope(&mut self) {
        self.locals.last_mut().unwrap().clear()
    }
}

pub struct Config {
    pub target: Target,
}

pub enum Target {
    Jit,
}

pub enum Output {
    Jit(i32),
}

pub fn compile(prog: Prog, config: Config) -> Output {
    let context = Context::create();
    let path = Path::new("target/rts.bc");
    let module = Module::parse_bitcode_from_path(&path, &context).unwrap();
    let builder = context.create_builder();

    module
        .get_function("noop")
        .unwrap()
        .set_linkage(Linkage::Internal);

    let term_type = context.opaque_struct_type("Term");
    term_type.set_body(
        &[
            BasicTypeEnum::PointerType(context.ptr_type(AddressSpace::from(0))),
            BasicTypeEnum::PointerType(context.ptr_type(AddressSpace::from(0))),
            BasicTypeEnum::IntType(context.i32_type()),
            BasicTypeEnum::IntType(context.i16_type()),
            BasicTypeEnum::IntType(context.i16_type()),
        ],
        false,
    );

    let fun_type = context.void_type().fn_type(
        &[BasicMetadataTypeEnum::PointerType(
            context.ptr_type(AddressSpace::from(0)),
        )],
        false,
    );

    let mut unit = Unit {
        context: &context,
        module,
        builder,
        term_type,
        fun_type,
        locals: Vec::new(),
    };

    prog.globals
        .into_iter()
        .for_each(|global| compile_global(&mut unit, global));

    prog.funs
        .into_iter()
        .for_each(|fun| compile_fun(&mut unit, fun));

    compile_main(&mut unit, prog.main);

    let s = unit.module.print_to_string().to_string();
    println!("{}", s);

    if let Err(e) = unit.module.verify() {
        panic!("LLVM verify error:\n{}", e.to_string());
    };

    match config.target {
        Target::Jit => Output::Jit(jit(&unit)),
    }
}

fn compile_global(unit: &mut Unit, global: Global) {
    let noop = unit.module.get_function("noop").unwrap();
    add_global(unit, noop, global.name, global.symbol, global.arity);
}

fn compile_fun(unit: &mut Unit, fun: Fun) {
    let function = unit
        .module
        .add_function("", unit.fun_type, Some(Linkage::Internal));
    let block = unit.context.append_basic_block(function, "start");
    unit.builder.position_at_end(block);

    unit.clear_locals();
    unit.add_scope();

    let arg = function.get_first_param().unwrap().into_pointer_value();
    unit.define(fun.arg_name, arg);

    compile_block(unit, fun.block);
}

fn compile_main(unit: &mut Unit, main: Block) {
    let main_fun_type = unit.context.i32_type().fn_type(&[], false);
    let main_fun = unit.module.add_function("main", main_fun_type, None);
    let block = unit.context.append_basic_block(main_fun, "start");
    unit.builder.position_at_end(block);

    unit.clear_locals();
    unit.add_scope();

    compile_block(unit, main);
}

fn compile_block(unit: &mut Unit, block: Block) {
    block.into_iter().for_each(|op| compile_op(unit, op));
}

fn compile_op(unit: &mut Unit<'_>, op: Op) {
    match op {
        Op::LoadGlobal(LoadGlobal { name, global }) => {
            let global = unit.module.get_global(global).unwrap();
            let global = unit
                .builder
                .build_load(unit.term_type, global.as_pointer_value(), "")
                .unwrap();
            let alloca = unit.builder.build_alloca(unit.term_type, "").unwrap();
            unit.builder.build_store(alloca, global).unwrap();

            unit.define(name, alloca);
        }
        Op::LoadArg(LoadArg { name, var, index }) => {
            let term = unit.lookup(var);

            let args_field = unit
                .builder
                .build_load(unit.term_type, term, "")
                .unwrap()
                .into_pointer_value();
            let arg_index = unit.context.i64_type().const_int(index, false);
            let arg_ptr = unsafe {
                unit.builder
                    .build_gep(unit.term_type, args_field, &[arg_index], "")
                    .unwrap()
            };
            let arg = unit
                .builder
                .build_load(unit.term_type, arg_ptr, "")
                .unwrap()
                .into_pointer_value();
            let arg_alloca = unit.builder.build_alloca(unit.term_type, "").unwrap();
            unit.builder.build_store(arg, arg_alloca).unwrap();

            unit.define(name, arg_alloca);
        }
        Op::NewApp(_) => todo!(),
        Op::NewPartial(_) => todo!(),
        Op::ApplyPartial(_) => todo!(),
        Op::Copy(_) => todo!(),
        Op::FreeArgs(_) => todo!(),
        Op::FreeTerm(_) => todo!(),
        Op::Eval(_) => todo!(),
        Op::Return(_) => todo!(),
        Op::ReturnSymbol(ReturnSymbol { var }) => {
            let term = unit.lookup(var);
            let term_load = unit
                .builder
                .build_load(unit.term_type, term, "")
                .unwrap()
                .into_struct_value();
            let symbol = unit.builder.build_extract_value(term_load, 2, "").unwrap();
            unit.builder.build_return(Some(&symbol)).unwrap();
        }
        Op::Switch(_) => todo!(),
    }
}

fn jit(unit: &Unit) -> i32 {
    let engine = unit
        .module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    type MainFun = unsafe extern "C" fn() -> i32;
    let main_fun = unsafe { engine.get_function::<MainFun>("main") }.unwrap();
    return unsafe { main_fun.call() };
}

fn add_global(unit: &mut Unit, fun: FunctionValue, name: Name, symbol: Symbol, arity: Arity) {
    let term_type = unit.term_type;

    let struct_val = term_type.const_named_struct(&[
        BasicValueEnum::PointerValue(fun.as_global_value().as_pointer_value()),
        BasicValueEnum::PointerValue(unit.context.ptr_type(AddressSpace::from(0)).const_null()),
        BasicValueEnum::IntValue(unit.context.i32_type().const_int(symbol.into(), false)),
        BasicValueEnum::IntValue(unit.context.i16_type().const_int(arity.into(), false)),
        BasicValueEnum::IntValue(unit.context.i16_type().const_int(arity.into(), false)),
    ]);

    let global = unit
        .module
        .add_global(term_type, Some(AddressSpace::from(0)), &name.to_string());
    global.set_constant(true);
    global.set_linkage(Linkage::Internal);
    global.set_initializer(&struct_val);
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($prog:expr, $expected:expr) => {
            let Output::Jit(result) = compile(
                $prog,
                Config {
                    target: Target::Jit,
                },
            );
            assert_eq!(result, $expected);
        };
    }

    #[test]
    fn test_return_symbol() {
        test!(
            Prog {
                globals: vec![Global {
                    name: "True",
                    symbol: 1,
                    arity: 0,
                }],
                funs: vec![],
                main: vec![
                    Op::LoadGlobal(LoadGlobal {
                        name: "True",
                        global: "True",
                    }),
                    Op::ReturnSymbol(ReturnSymbol { var: "True" }),
                ],
            },
            1
        );
    }
}
