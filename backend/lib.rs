use std::path::Path;

use inkwell::{
  builder::Builder,
  context::Context,
  module::{Linkage, Module},
  types::BasicTypeEnum,
  values::{BasicValueEnum, FunctionValue},
  AddressSpace, OptimizationLevel,
};

type Key = u64;

type Symbol = u32;

type Arity = u16;

type Index = u64;

pub struct Project {
  context: Context,
}

pub struct Unit<'ctx> {
  context: &'ctx Context,
  module: Module<'ctx>,
  builder: Builder<'ctx>,
}

#[no_mangle]
pub extern "C" fn new_project() -> Box<Project> {
  Box::new(Project {
    context: Context::create(),
  })
}

#[no_mangle]
pub extern "C" fn free_project(_project: Box<Project>) {}

#[no_mangle]
pub extern "C" fn new_unit<'ctx>(project: &'ctx Project) -> Box<Unit<'ctx>> {
  let path = Path::new("lib/rts.bc");
  let rts = Module::parse_bitcode_from_path(&path, &project.context).unwrap();
  for fun in rts.get_functions() {
    fun.set_linkage(Linkage::Internal);
  }

  let unit = Unit {
    context: &project.context,
    module: rts,
    builder: project.context.create_builder(),
  };

  let term_type = unit.context.opaque_struct_type("Term");
  term_type.set_body(
    &[
      BasicTypeEnum::PointerType(unit.context.ptr_type(AddressSpace::from(0))),
      BasicTypeEnum::PointerType(unit.context.ptr_type(AddressSpace::from(0))),
      BasicTypeEnum::IntType(unit.context.i32_type()),
      BasicTypeEnum::IntType(unit.context.i16_type()),
      BasicTypeEnum::IntType(unit.context.i16_type()),
    ],
    false,
  );

  Box::new(unit)
}

#[no_mangle]
pub extern "C" fn free_unit(_unit: Box<Unit>) {}

#[no_mangle]
pub extern "C" fn print_unit(unit: &Unit<'_>) {
  println!("{}", unit.module.to_string());
}

#[no_mangle]
pub extern "C" fn add_main(unit: &Unit<'_>) {
  let main_fun_type = unit.context.i32_type().fn_type(&[], false);
  let function = unit.module.add_function("main", main_fun_type, None);
  let block = unit.context.append_basic_block(function, "start");
  unit.builder.position_at_end(block);
  unit
    .builder
    .build_return(Some(&unit.context.i32_type().const_int(42, false)))
    .unwrap();
}

#[no_mangle]
pub extern "C" fn add_data(unit: &mut Unit<'_>, name: Key, symbol: Symbol, arity: Arity) {
  let noop = unit.module.get_function("noop").unwrap();
  add_global(unit, noop, name, symbol, arity);
}

fn add_global(unit: &mut Unit<'_>, fun: FunctionValue, name: Key, symbol: Symbol, arity: Arity) {
  let term_type = unit.module.get_struct_type("Term").unwrap();

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

#[no_mangle]
pub extern "C" fn jit(unit: &Unit<'_>) -> i32 {
  let engine = unit
    .module
    .create_jit_execution_engine(OptimizationLevel::None)
    .unwrap();
  type MainFun = unsafe extern "C" fn() -> i32;
  let main_fun = unsafe { engine.get_function::<MainFun>("main") }.unwrap();
  return unsafe { main_fun.call() };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_jit() {
    let project = new_project();
    let mut unit = new_unit(&project);
    add_data(&mut unit, 0, 0, 0);
    add_main(&unit);
    if let Err(e) = unit.module.verify() {
      eprintln!("{}", e.to_string());
    };
    let result = jit(&unit);
    assert_eq!(result, 42);
  }
}
