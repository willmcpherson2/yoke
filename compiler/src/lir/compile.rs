use super::*;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::{Linkage, Module},
    passes::PassBuilderOptions,
    targets::{FileType, InitializationConfig, Target, TargetMachine, TargetMachineOptions},
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, StructType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};
use std::{collections::HashMap, path::Path};

const RTS_BC: &[u8] = include_bytes!("../../../target/rts.bc");

#[derive(Debug)]
pub struct Config {
    pub mode: Mode,
    pub opt_level: OptLevel,
}

#[derive(Debug)]
pub enum Mode {
    Jit,
    Aot,
}

#[derive(Debug)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

#[derive(Debug)]
pub enum Output {
    ExitCode(i32),
    Binary,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Jit,
            opt_level: OptLevel::O0,
        }
    }
}

pub fn compile(program: &Program, config: Config) -> Output {
    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let options = TargetMachineOptions::new().set_level(OptimizationLevel::None);
    let machine = target
        .create_target_machine_from_options(&triple, options)
        .unwrap();

    let context = Context::create();
    let buffer = MemoryBuffer::create_from_memory_range(RTS_BC, "main");
    let module = Module::parse_bitcode_from_buffer(&buffer, &context).unwrap();
    let builder = context.create_builder();

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
        program,
        machine,
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

    program.iter().for_each(|(name, global)| {
        if name != "main" {
            define_const(name, global, &mut unit);
        }
    });

    program.iter().for_each(|(name, global)| match global {
        Global::Fun { block, .. } if name == "main" => define_main(block, &mut unit),
        Global::Fun { block, .. } => define_fun(name, block, &mut unit),
        _ => {}
    });

    opt(&unit);

    if let Err(e) = unit.module.verify() {
        unit.print();
        panic!("LLVM verify error:\n{}", e.to_string());
    };

    match unit.config.mode {
        Mode::Jit => Output::ExitCode(jit(&unit)),
        Mode::Aot => {
            binary(&unit);
            Output::Binary
        }
    }
}

#[derive(Debug)]
struct Unit<'ctx> {
    config: Config,
    program: &'ctx Program,
    machine: TargetMachine,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    term_type: StructType<'ctx>,
    fun_type: FunctionType<'ctx>,
    fun: Option<FunctionValue<'ctx>>,
    block: Option<BasicBlock<'ctx>>,
    arg: Option<PointerValue<'ctx>>,
    locals: Vec<HashMap<Name, PointerValue<'ctx>>>,
}

fn define_const(name: &str, global: &Global, unit: &mut Unit) {
    match global {
        Global::Const { arity, symbol } => {
            let noop = unit.module.get_function("noop").unwrap();
            add_global(noop, format!("term_{name}"), *symbol, *arity, unit);
        }
        Global::Fun { arity, .. } => {
            let fun = unit.module.add_function(
                &format!("fun_{name}"),
                unit.fun_type,
                Some(Linkage::Internal),
            );
            add_global(fun, format!("term_{name}"), 0, *arity, unit);
        }
    }
}

fn define_main(main: &Block, unit: &mut Unit) {
    let main_fun_type = unit.context.i32_type().fn_type(&[], false);
    let fun = unit.module.add_function("main", main_fun_type, None);
    unit.fun = Some(fun);

    let block = unit.context.append_basic_block(fun, "start");
    unit.block = Some(block);
    unit.builder.position_at_end(block);

    unit.clear_locals();
    unit.add_scope();

    compile_block(main, unit);
}

fn define_fun(name: &str, block: &Block, unit: &mut Unit) {
    let fun = unit.module.get_function(&format!("fun_{name}")).unwrap();
    unit.fun = Some(fun);

    let basic_block = unit.context.append_basic_block(fun, "start");
    unit.block = Some(basic_block);
    unit.builder.position_at_end(basic_block);

    unit.clear_locals();
    unit.add_scope();

    let arg = fun.get_first_param().unwrap().into_pointer_value();
    unit.arg = Some(arg);
    unit.define("self".to_string(), arg);

    compile_block(block, unit);
}

fn compile_block(block: &Block, unit: &mut Unit) {
    block.iter().for_each(|op| compile_op(op, unit));
}

fn compile_op(op: &Op, unit: &mut Unit) {
    match op {
        Op::LoadGlobal { global } => {
            let global_value = unit
                .module
                .get_global(&format!("term_{}", &global))
                .unwrap();
            let global_load = unit
                .builder
                .build_load(unit.term_type, global_value.as_pointer_value(), "")
                .unwrap();
            let alloca = unit.builder.build_alloca(unit.term_type, "").unwrap();
            unit.builder.build_store(alloca, global_load).unwrap();

            unit.define(global.clone(), alloca);
        }
        Op::LoadArg { name, var, index } => {
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
            let arg_index = unit.context.i64_type().const_int(*index, false);
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

            unit.define(name.clone(), arg_alloca);
        }
        Op::NewApp {
            name,
            var,
            ref args,
        } => compile_apply_call(name.clone(), "new_app", var, args, unit),
        Op::NewPartial {
            name,
            var,
            ref args,
        } => compile_apply_call(name.clone(), "new_partial", var, args, unit),
        Op::ApplyPartial {
            name,
            var,
            ref args,
        } => compile_apply_call(name.clone(), "apply_partial", var, args, unit),
        Op::Copy { name, var } => {
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

            unit.define(name.clone(), dest);
        }
        Op::Eval { var } => {
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
        }
        Op::FreeArgs { var } => {
            let term = unit.lookup(var);
            let free_args = unit.module.get_function("free_args").unwrap();
            unit.builder
                .build_call(free_args, &[BasicMetadataValueEnum::PointerValue(term)], "")
                .unwrap();
        }
        Op::FreeTerm { var } => {
            let term = unit.lookup(var);
            let free_term = unit.module.get_function("free_term").unwrap();
            unit.builder
                .build_call(free_term, &[BasicMetadataValueEnum::PointerValue(term)], "")
                .unwrap();
        }
        Op::ReturnSymbol { var } => {
            let term = unit.lookup(var);
            let term_load = unit
                .builder
                .build_load(unit.term_type, term, "")
                .unwrap()
                .into_struct_value();
            let symbol = unit.builder.build_extract_value(term_load, 2, "").unwrap();
            unit.builder.build_return(Some(&symbol)).unwrap();
        }
        Op::Return { var } => {
            let term = unit.lookup(var);
            let term_load = unit.builder.build_load(unit.term_type, term, "").unwrap();
            unit.builder
                .build_store(unit.arg.unwrap(), term_load)
                .unwrap();
            unit.builder.build_return(None).unwrap();
        }
        Op::Switch { var, ref cases } => {
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
                    let Global::Const { symbol, .. } = unit.program.get(&case.global).unwrap()
                    else {
                        panic!()
                    };
                    let symbol = unit.context.i32_type().const_int(*symbol as u64, false);
                    let block = unit.context.append_basic_block(unit.fun.unwrap(), "");
                    unit.builder.position_at_end(block);
                    unit.clear_scope();
                    compile_block(&case.block, unit);
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
        Op::Todo => {
            let todo = unit.module.get_function("todo").unwrap();
            unit.builder.build_call(todo, &[], "").unwrap();
        }
    }
}

fn compile_apply_call(name: Name, fun_name: &str, var: &str, args: &[String], unit: &mut Unit) {
    let term = unit.lookup(var);
    let length_constant = unit.context.i64_type().const_int(args.len() as u64, false);
    let args_type = unit.term_type.array_type(args.len() as u32);
    let args_alloca = unit.builder.build_alloca(args_type, "").unwrap();
    for (i, arg) in args.iter().enumerate() {
        let arg_local = unit.lookup(arg);
        let arg_load = unit
            .builder
            .build_load(unit.term_type, arg_local, "")
            .unwrap();
        let indexes = [
            unit.context.i64_type().const_int(0, false),
            unit.context.i64_type().const_int(i as u64, false),
        ];
        let arg_gep = unsafe {
            unit.builder
                .build_gep(args_type, args_alloca, &indexes, "")
                .unwrap()
        };
        unit.builder.build_store(arg_gep, arg_load).unwrap();
    }

    let fun = unit.module.get_function(fun_name).unwrap();
    unit.builder
        .build_call(
            fun,
            &[term.into(), args_alloca.into(), length_constant.into()],
            "",
        )
        .unwrap();

    unit.define(name, term);
}

fn add_global(fun: FunctionValue, name: Name, symbol: Symbol, arity: Arity, unit: &mut Unit) {
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
        .add_global(term_type, Some(AddressSpace::from(0)), &name);
    global.set_constant(true);
    global.set_linkage(Linkage::Internal);
    global.set_initializer(&struct_val);
}

fn opt(unit: &Unit) {
    let pass = match unit.config.opt_level {
        OptLevel::O0 => return,
        OptLevel::O1 => "default<O1>",
        OptLevel::O2 => "default<O2>",
        OptLevel::O3 => "default<O3>",
    };
    unit.module
        .run_passes(pass, &unit.machine, PassBuilderOptions::create())
        .unwrap();
}

fn jit(unit: &Unit) -> i32 {
    let engine = unit
        .module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    type MainFun = unsafe extern "C" fn() -> i32;
    let main_fun = unsafe { engine.get_function::<MainFun>("main") }.unwrap();
    unsafe { main_fun.call() }
}

fn binary(unit: &Unit) {
    unit.machine
        .write_to_file(&unit.module, FileType::Object, Path::new("main.o"))
        .unwrap();
}

impl<'ctx> Unit<'ctx> {
    fn add_scope(&mut self) {
        self.locals.push(HashMap::new())
    }

    fn define(&mut self, name: Name, value: PointerValue<'ctx>) {
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

    fn print(&self) {
        let s = self.module.print_to_string().to_string();
        println!("{}", s);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($prog:expr, $expected:expr) => {
            let Output::ExitCode(result) = compile(&$prog, Config::default()) else {
                panic!()
            };
            assert_eq!(result, $expected);
        };
    }

    #[test]
    fn test_return_symbol() {
        test!(
            HashMap::from([
                (
                    "True".to_string(),
                    Global::Const {
                        arity: 0,
                        symbol: 1,
                    }
                ),
                (
                    "main".to_string(),
                    Global::Fun {
                        arity: 0,
                        block: vec![
                            Op::LoadGlobal {
                                global: "True".to_string(),
                            },
                            Op::ReturnSymbol {
                                var: "True".to_string()
                            },
                        ]
                    }
                )
            ]),
            1
        );
    }

    #[test]
    fn test_copy() {
        test!(
            HashMap::from([
                (
                    "True".to_string(),
                    Global::Const {
                        arity: 0,
                        symbol: 1,
                    },
                ),
                (
                    "main".to_string(),
                    Global::Fun {
                        arity: 0,
                        block: vec![
                            Op::LoadGlobal {
                                global: "True".to_string(),
                            },
                            Op::Copy {
                                name: "x".to_string(),
                                var: "True".to_string(),
                            },
                            Op::FreeTerm {
                                var: "x".to_string()
                            },
                            Op::ReturnSymbol {
                                var: "x".to_string()
                            },
                        ]
                    }
                )
            ]),
            1
        );
    }

    #[test]
    fn test_id() {
        test!(
            HashMap::from([
                (
                    "True".to_string(),
                    Global::Const {
                        arity: 0,
                        symbol: 1,
                    },
                ),
                (
                    "id".to_string(),
                    Global::Fun {
                        arity: 1,
                        block: vec![
                            Op::LoadArg {
                                name: "x".to_string(),
                                var: "self".to_string(),
                                index: 0,
                            },
                            Op::FreeArgs {
                                var: "self".to_string()
                            },
                            Op::Eval {
                                var: "x".to_string(),
                            },
                            Op::Return {
                                var: "x".to_string()
                            },
                        ],
                    }
                ),
                (
                    "main".to_string(),
                    Global::Fun {
                        arity: 0,
                        block: vec![
                            Op::LoadGlobal {
                                global: "id".to_string(),
                            },
                            Op::LoadGlobal {
                                global: "True".to_string(),
                            },
                            Op::NewApp {
                                name: "result".to_string(),
                                var: "id".to_string(),
                                args: vec!["True".to_string()]
                            },
                            Op::Eval {
                                var: "result".to_string(),
                            },
                            Op::ReturnSymbol {
                                var: "result".to_string()
                            },
                        ],
                    }
                ),
            ]),
            1
        );
    }

    #[test]
    fn test_switch() {
        test!(
            HashMap::from([
                (
                    "True".to_string(),
                    Global::Const {
                        arity: 0,
                        symbol: 1,
                    },
                ),
                (
                    "False".to_string(),
                    Global::Const {
                        arity: 0,
                        symbol: 2,
                    },
                ),
                (
                    "main".to_string(),
                    Global::Fun {
                        arity: 0,
                        block: vec![
                            Op::LoadGlobal {
                                global: "True".to_string(),
                            },
                            Op::LoadGlobal {
                                global: "False".to_string(),
                            },
                            Op::Switch {
                                var: "True".to_string(),
                                cases: vec![
                                    Case {
                                        global: "True".to_string(),
                                        block: vec![Op::ReturnSymbol {
                                            var: "False".to_string()
                                        }]
                                    },
                                    Case {
                                        global: "False".to_string(),
                                        block: vec![Op::ReturnSymbol {
                                            var: "True".to_string()
                                        }]
                                    },
                                ],
                            },
                        ],
                    }
                ),
            ]),
            2
        );
    }
}
