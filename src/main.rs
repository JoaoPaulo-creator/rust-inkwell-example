use std::env;
use std::fs;

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;

use crate::codegen::CodeGen;
use crate::error::CompileError;
use crate::parser::Parser;

fn main() {
    if let Err(err) = run_compiler() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run_compiler() -> Result<(), CompileError> {
    // Read source file path from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(CompileError::Io("No source file provided".to_string()));
    }
    let filename = &args[1];
    let source = fs::read_to_string(filename)
        .map_err(|e| CompileError::Io(format!("Failed to read file: {}", e)))?;
    // Lexical analysis: produce tokens
    let tokens = lexer::lex(&source)?;
    // Parsing: convert tokens into AST
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    // Code generation: produce LLVM IR and JIT compile it
    let context = inkwell::context::Context::create();
    let module = context.create_module("my_module");
    let mut codegen = CodeGen::new(&context, module);
    codegen.compile(&ast)?;
    // Output the generated LLVM IR
    println!("Generated LLVM IR:\n{}", codegen.module.print_to_string());
    // JIT execute the compiled code (the 'main' function)
    let execution_engine = codegen
        .module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| CompileError::Codegen(format!("Failed to create JIT engine: {}", e)))?;
    unsafe {
        type MainFunc = unsafe extern "C" fn() -> i32;
        let main_func = execution_engine
            .get_function::<MainFunc>("main")
            .map_err(|_| CompileError::Codegen("Could not find 'main' function".to_string()))?;
        let result = main_func.call();
        println!("Program result: {}", result);
    }
    Ok(())
}
