use crate::error::CompileError;
use std::env;
use std::fs;

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;

use ast::Program;
use codegen::CodeGen;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), CompileError> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| CompileError::Io("No input file specified".into()))?;
    let src = fs::read_to_string(path).map_err(|e| CompileError::Io(e.to_string()))?;

    // lex & parse
    let tokens = lexer::lex(&src)?;
    let mut parser = parser::Parser::new(tokens);
    let prog: Program = parser.parse_program()?;

    // codegen
    let ctx = inkwell::context::Context::create();
    let module = ctx.create_module("toy");
    let mut cg = CodeGen::new(&ctx, module);
    cg.compile_program(&prog)?;

    let ir = cg.module.print_to_string().to_string();
    std::fs::write("program.ll", ir)
        .map_err(|e| CompileError::Io(format!("Failed to write IR file: {}", e)))?;

    // JIT & run
    let ee = cg
        .module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| CompileError::Codegen(format!("{:?}", e)))?;
    unsafe {
        let main_fn = ee
            .get_function::<unsafe extern "C" fn() -> i32>("main")
            .map_err(|_| CompileError::Codegen("No main()".into()))?;
        println!(">>> Program returned: {}", main_fn.call());
    }
    Ok(())
}
