use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::{Linkage, Module},
    targets::{self, InitializationConfig},
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, StructType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub use inkwell::OptimizationLevel;

const RTS_BC: &[u8] = include_bytes!("../../target/release/deps/rts.bc");

#[derive(Debug)]
pub struct Config {
    pub target: Target,
    pub opt_level: OptimizationLevel,
}

#[derive(Debug)]
pub enum Target {
    Jit,
    Binary,
}

#[derive(Debug)]
pub enum Output {
    Jit(i32),
    Binary,
}

pub type Name = &'static str;

pub type Symbol = u32;

pub type Arity = u16;

#[derive(Debug)]
pub struct Prog {
    pub globals: Vec<Global>,
    pub funs: Vec<Fun>,
    pub main: Block,
}

impl Prog {
    pub fn compile(&self, config: Config) -> Output {
        let context = Context::create();
        let buffer = MemoryBuffer::create_from_memory_range(RTS_BC, "rts");
        let module = Module::parse_bitcode_from_buffer(&buffer, &context).unwrap();
        let builder = context.create_builder();

        let rts_fun_names = HashSet::from([
            c"noop",
            c"new_app",
            c"new_partial",
            c"apply_partial",
            c"copy",
            c"free_args",
            c"free_term",
        ]);
        let rts_funs = module
            .get_functions()
            .filter(|fun| rts_fun_names.contains(fun.get_name()));
        for fun in rts_funs {
            fun.set_linkage(Linkage::Internal)
        }

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
            config,
            context: &context,
            module,
            builder,
            term_type,
            fun_type,
            fun: None,
            block: None,
            arg: None,
            locals: Vec::new(),
        };

        self.globals
            .iter()
            .for_each(|global| global.compile(&mut unit));

        self.funs.iter().for_each(|fun| fun.compile(&mut unit));

        self.compile_main(&mut unit);

        if let Err(e) = unit.module.verify() {
            unit.print();
            panic!("LLVM verify error:\n{}", e.to_string());
        };

        match unit.config.target {
            Target::Jit => Output::Jit(unit.jit()),
            Target::Binary => {
                unit.binary();
                Output::Binary
            }
        }
    }

    fn compile_main(&self, unit: &mut Unit) {
        let main_fun_type = unit.context.i32_type().fn_type(&[], false);
        let fun = unit.module.add_function("main", main_fun_type, None);
        unit.fun = Some(fun);

        let block = unit.context.append_basic_block(fun, "start");
        unit.block = Some(block);
        unit.builder.position_at_end(block);

        unit.clear_locals();
        unit.add_scope();

        self.main.compile(unit);
    }
}

#[derive(Debug)]
pub struct Global {
    pub name: Name,
    pub symbol: Symbol,
    pub arity: Arity,
}

impl Global {
    fn compile(&self, unit: &mut Unit) {
        let noop = unit.module.get_function("noop").unwrap();
        unit.add_global(noop, self.name, self.symbol, self.arity);
    }
}

#[derive(Debug)]
pub struct Fun {
    pub name: Name,
    pub arg_name: Name,
    pub symbol: Symbol,
    pub arity: Arity,
    pub block: Block,
}

impl Fun {
    fn compile(&self, unit: &mut Unit) {
        let fun = unit
            .module
            .add_function("", unit.fun_type, Some(Linkage::Internal));
        unit.fun = Some(fun);

        let block = unit.context.append_basic_block(fun, "start");
        unit.block = Some(block);
        unit.builder.position_at_end(block);

        unit.clear_locals();
        unit.add_scope();

        let arg = fun.get_first_param().unwrap().into_pointer_value();
        unit.arg = Some(arg);
        unit.define(self.arg_name, arg);

        self.block.compile(unit);

        unit.add_global(fun, self.name, self.symbol, self.arity);
    }
}

#[derive(Debug)]
pub struct Block(pub Vec<Op>);

impl Block {
    fn compile(&self, unit: &mut Unit) {
        self.0.iter().for_each(|op| op.compile(unit));
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct LoadGlobal {
    pub name: Name,
    pub global: Name,
}

#[derive(Debug)]
pub struct LoadArg {
    pub name: Name,
    pub var: Name,
    pub index: u64,
}

#[derive(Debug)]
pub struct NewApp {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

#[derive(Debug)]
pub struct NewPartial {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

#[derive(Debug)]
pub struct ApplyPartial {
    pub name: Name,
    pub var: Name,
    pub args: Vec<Name>,
}

#[derive(Debug)]
pub struct Copy {
    pub name: Name,
    pub var: Name,
}

#[derive(Debug)]
pub struct FreeArgs {
    pub var: Name,
}

#[derive(Debug)]
pub struct FreeTerm {
    pub var: Name,
}

#[derive(Debug)]
pub struct Eval {
    pub name: Name,
    pub var: Name,
}

#[derive(Debug)]
pub struct Return {
    pub var: Name,
}

#[derive(Debug)]
pub struct ReturnSymbol {
    pub var: Name,
}

#[derive(Debug)]
pub struct Switch {
    pub var: Name,
    pub cases: Vec<Case>,
}

#[derive(Debug)]
pub struct Case {
    pub symbol: Symbol,
    pub block: Block,
}

impl Op {
    fn compile(&self, unit: &mut Unit) {
        match *self {
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

                let term_load = unit
                    .builder
                    .build_load(unit.term_type, term, "")
                    .unwrap()
                    .into_struct_value();
                let args_field = unit
                    .builder
                    .build_extract_value(term_load, 1, "")
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
                    .unwrap();
                let arg_alloca = unit.builder.build_alloca(unit.term_type, "").unwrap();
                unit.builder.build_store(arg_alloca, arg).unwrap();

                unit.define(name, arg_alloca);
            }
            Op::NewApp(NewApp {
                name,
                var,
                ref args,
            }) => unit.compile_apply_call(name, "new_app", var, args),
            Op::NewPartial(NewPartial {
                name,
                var,
                ref args,
            }) => unit.compile_apply_call(name, "new_partial", var, args),
            Op::ApplyPartial(ApplyPartial {
                name,
                var,
                ref args,
            }) => unit.compile_apply_call(name, "apply_partial", var, args),
            Op::Copy(Copy { name, var }) => {
                let dest = unit.builder.build_alloca(unit.term_type, "").unwrap();
                let src = unit.lookup(var);
                let copy = unit.module.get_function("copy").unwrap();
                unit.builder
                    .build_call(
                        copy,
                        &[
                            BasicMetadataValueEnum::PointerValue(dest),
                            BasicMetadataValueEnum::PointerValue(src),
                        ],
                        "",
                    )
                    .unwrap();

                unit.define(name, dest);
            }
            Op::FreeArgs(FreeArgs { var }) => {
                let term = unit.lookup(var);
                let free_args = unit.module.get_function("free_args").unwrap();
                unit.builder
                    .build_call(free_args, &[BasicMetadataValueEnum::PointerValue(term)], "")
                    .unwrap();
            }
            Op::FreeTerm(FreeTerm { var }) => {
                let term = unit.lookup(var);
                let free_term = unit.module.get_function("free_term").unwrap();
                unit.builder
                    .build_call(free_term, &[BasicMetadataValueEnum::PointerValue(term)], "")
                    .unwrap();
            }
            Op::Eval(Eval { name, var }) => {
                let term = unit.lookup(var);
                let term_load = unit
                    .builder
                    .build_load(unit.term_type, term, "")
                    .unwrap()
                    .into_struct_value();
                let fun = unit
                    .builder
                    .build_extract_value(term_load, 0, "")
                    .unwrap()
                    .into_pointer_value();
                unit.builder
                    .build_indirect_call(unit.fun_type, fun, &[term.into()], "")
                    .unwrap();

                unit.define(name, term);
            }
            Op::Return(Return { var }) => {
                let term = unit.lookup(var);
                let term_load = unit.builder.build_load(unit.term_type, term, "").unwrap();
                unit.builder
                    .build_store(unit.arg.unwrap(), term_load)
                    .unwrap();
                unit.builder.build_return(None).unwrap();
            }
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
            Op::Switch(Switch { var, ref cases }) => {
                let term = unit.lookup(var);
                let term_load = unit
                    .builder
                    .build_load(unit.term_type, term, "")
                    .unwrap()
                    .into_struct_value();
                let symbol = unit
                    .builder
                    .build_extract_value(term_load, 2, "")
                    .unwrap()
                    .into_int_value();

                unit.add_scope();

                let cases = cases
                    .iter()
                    .map(|case| {
                        let symbol = unit.context.i32_type().const_int(case.symbol.into(), false);
                        let block = unit.context.append_basic_block(unit.fun.unwrap(), "");
                        unit.builder.position_at_end(block);
                        unit.clear_scope();
                        case.block.compile(unit);
                        (symbol, block)
                    })
                    .collect::<Vec<_>>();

                let default_case = unit
                    .context
                    .append_basic_block(unit.fun.unwrap(), "default");
                unit.builder.position_at_end(default_case);
                unit.builder.build_unreachable().unwrap();

                unit.builder.position_at_end(unit.block.unwrap());
                unit.builder
                    .build_switch(symbol, default_case, &cases)
                    .unwrap();
            }
        }
    }
}

#[derive(Debug)]
struct Unit<'ctx> {
    config: Config,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    term_type: StructType<'ctx>,
    fun_type: FunctionType<'ctx>,
    fun: Option<FunctionValue<'ctx>>,
    block: Option<BasicBlock<'ctx>>,
    arg: Option<PointerValue<'ctx>>,
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
                return *local;
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

    fn add_global(&mut self, fun: FunctionValue, name: Name, symbol: Symbol, arity: Arity) {
        let term_type = self.term_type;

        let struct_val = term_type.const_named_struct(&[
            BasicValueEnum::PointerValue(fun.as_global_value().as_pointer_value()),
            BasicValueEnum::PointerValue(self.context.ptr_type(AddressSpace::from(0)).const_null()),
            BasicValueEnum::IntValue(self.context.i32_type().const_int(symbol.into(), false)),
            BasicValueEnum::IntValue(self.context.i16_type().const_int(arity.into(), false)),
            BasicValueEnum::IntValue(self.context.i16_type().const_int(arity.into(), false)),
        ]);

        let global = self
            .module
            .add_global(term_type, Some(AddressSpace::from(0)), name);
        global.set_constant(true);
        global.set_linkage(Linkage::Internal);
        global.set_initializer(&struct_val);
    }

    fn jit(&self) -> i32 {
        let engine = self
            .module
            .create_jit_execution_engine(self.config.opt_level)
            .unwrap();
        type MainFun = unsafe extern "C" fn() -> i32;
        let main_fun = unsafe { engine.get_function::<MainFun>("main") }.unwrap();
        unsafe { main_fun.call() }
    }

    fn binary(&self) {
        targets::Target::initialize_all(&InitializationConfig::default());
        let target_triple = inkwell::targets::TargetMachine::get_default_triple();
        let target = targets::Target::from_triple(&target_triple)
            .expect("Failed to create target from triple");
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.config.opt_level,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .unwrap();
        target_machine
            .write_to_file(
                &self.module,
                inkwell::targets::FileType::Object,
                Path::new("main.o"),
            )
            .unwrap();
    }

    fn print(&self) {
        let s = self.module.print_to_string().to_string();
        println!("{}", s);
    }

    fn compile_apply_call(
        &mut self,
        name: &'ctx str,
        fun_name: &'static str,
        var: &str,
        args: &Vec<&str>,
    ) {
        let term = self.lookup(var);
        let length_constant = self.context.i64_type().const_int(args.len() as u64, false);
        let args_type = self.term_type.array_type(args.len() as u32);
        let args_alloca = self.builder.build_alloca(args_type, "").unwrap();
        for (i, arg) in args.iter().enumerate() {
            let arg_local = self.lookup(arg);
            let arg_load = self
                .builder
                .build_load(self.term_type, arg_local, "")
                .unwrap();
            let indexes = [
                self.context.i64_type().const_int(0, false),
                self.context.i64_type().const_int(i as u64, false),
            ];
            let arg_gep = unsafe {
                self.builder
                    .build_gep(args_type, args_alloca, &indexes, "")
                    .unwrap()
            };
            self.builder.build_store(arg_gep, arg_load).unwrap();
        }

        let fun = self.module.get_function(fun_name).unwrap();
        self.builder
            .build_call(
                fun,
                &[term.into(), args_alloca.into(), length_constant.into()],
                "",
            )
            .unwrap();

        self.define(name, term);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($prog:expr, $expected:expr) => {
            let Output::Jit(result) = $prog.compile(Config {
                target: Target::Jit,
                opt_level: OptimizationLevel::None,
            }) else {
                panic!()
            };
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
                main: Block(vec![
                    Op::LoadGlobal(LoadGlobal {
                        name: "True",
                        global: "True",
                    }),
                    Op::ReturnSymbol(ReturnSymbol { var: "True" }),
                ]),
            },
            1
        );
    }

    #[test]
    fn test_copy() {
        test!(
            Prog {
                globals: vec![Global {
                    name: "True",
                    symbol: 1,
                    arity: 0,
                }],
                funs: vec![],
                main: Block(vec![
                    Op::LoadGlobal(LoadGlobal {
                        name: "True",
                        global: "True",
                    }),
                    Op::Copy(Copy {
                        name: "x",
                        var: "True",
                    }),
                    Op::FreeTerm(FreeTerm { var: "x" }),
                    Op::ReturnSymbol(ReturnSymbol { var: "x" }),
                ]),
            },
            1
        );
    }

    #[test]
    fn test_id() {
        test!(
            Prog {
                globals: vec![Global {
                    name: "True",
                    symbol: 1,
                    arity: 0,
                }],
                funs: vec![Fun {
                    name: "id",
                    arg_name: "self",
                    symbol: 2,
                    arity: 1,
                    block: Block(vec![
                        Op::LoadArg(LoadArg {
                            name: "x",
                            var: "self",
                            index: 0,
                        }),
                        Op::FreeArgs(FreeArgs { var: "self" }),
                        Op::Eval(Eval {
                            name: "x",
                            var: "x",
                        }),
                        Op::Return(Return { var: "x" }),
                    ]),
                }],
                main: Block(vec![
                    Op::LoadGlobal(LoadGlobal {
                        name: "id",
                        global: "id",
                    }),
                    Op::LoadGlobal(LoadGlobal {
                        name: "True",
                        global: "True",
                    }),
                    Op::NewApp(NewApp {
                        name: "result",
                        var: "id",
                        args: vec!["True"]
                    }),
                    Op::Eval(Eval {
                        name: "result",
                        var: "result",
                    }),
                    Op::ReturnSymbol(ReturnSymbol { var: "result" }),
                ]),
            },
            1
        );
    }

    #[test]
    fn test_switch() {
        test!(
            Prog {
                globals: vec![
                    Global {
                        name: "True",
                        symbol: 1,
                        arity: 0,
                    },
                    Global {
                        name: "False",
                        symbol: 2,
                        arity: 0,
                    },
                ],
                funs: vec![],
                main: Block(vec![
                    Op::LoadGlobal(LoadGlobal {
                        name: "True",
                        global: "True",
                    }),
                    Op::LoadGlobal(LoadGlobal {
                        name: "False",
                        global: "False",
                    }),
                    Op::Switch(Switch {
                        var: "True",
                        cases: vec![
                            Case {
                                symbol: 1,
                                block: Block(vec![Op::ReturnSymbol(ReturnSymbol { var: "False" })])
                            },
                            Case {
                                symbol: 2,
                                block: Block(vec![Op::ReturnSymbol(ReturnSymbol { var: "True" })])
                            },
                        ],
                    }),
                ]),
            },
            2
        );
    }
}
