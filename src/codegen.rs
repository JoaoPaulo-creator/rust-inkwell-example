use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{IntValue, PointerValue};
use std::collections::HashMap;

use crate::ast::{BinOp, Expr, Statement};
use crate::error::CompileError;

/// CodeGen generates LLVM IR from the AST using Inkwell.
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    i32_type: IntType<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        CodeGen {
            context,
            module,
            builder,
            variables: HashMap::new(),
            i32_type,
        }
    }

    /// Compile a list of Statements into LLVM IR (inside a 'main' function).
    pub fn compile(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        // Create a function `int main()` in the module
        let fn_type = self.i32_type.fn_type(&[], false);
        let main_func = self.module.add_function("main", fn_type, None);
        let entry_block = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry_block);

        // Allocate stack space for each variable used in the program
        for stmt in statements {
            if !self.variables.contains_key(&stmt.name) {
                let ptr = self
                    .builder
                    .build_alloca(self.i32_type, &stmt.name)
                    .map_err(|e| {
                        CompileError::Codegen(format!(
                            "Failed to allocate variable {}: {:?}",
                            stmt.name, e
                        ))
                    })?;
                self.variables.insert(stmt.name.clone(), ptr);
            }
        }

        // Generate code for each statement
        let mut last_value: Option<IntValue> = None;
        for stmt in statements {
            // Compile the right-hand side expression to an IntValue
            let value = self.compile_expr(&stmt.value)?;
            // Store the result into the variable's memory slot
            let var_ptr = self.variables.get(&stmt.name).ok_or_else(|| {
                CompileError::Codegen(format!("Variable {} not found", stmt.name))
            })?;
            self.builder.build_store(*var_ptr, value).map_err(|e| {
                CompileError::Codegen(format!("Failed to store variable {}: {:?}", stmt.name, e))
            })?;
            last_value = Some(value);
        }

        // Determine what value to return from main
        let return_val = if let Some(val) = last_value {
            val // return the last computed value
        } else {
            // No statements, return 0 by default
            self.i32_type.const_int(0, false)
        };
        // Build the return instruction
        self.builder
            .build_return(Some(&return_val))
            .map_err(|e| CompileError::Codegen(format!("Failed to build return: {:?}", e)))?;
        Ok(())
    }

    /// Compile an Expr AST node to an LLVM IntValue.
    fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, CompileError> {
        match expr {
            Expr::Number(n) => {
                // Generate a constant i32 value
                let const_val = if *n >= 0 {
                    self.i32_type.const_int(*n as u64, false)
                } else {
                    // For negative numbers, use sign extension
                    self.i32_type.const_int((*n as i64) as u64, true)
                };
                Ok(const_val)
            }
            Expr::Variable(name) => {
                // Load the value from the variable's pointer
                let var_ptr = self.variables.get(name).ok_or_else(|| {
                    CompileError::Codegen(format!("Use of undefined variable {}", name))
                })?;
                let loaded_val = self
                    .builder
                    .build_load(self.i32_type, *var_ptr, name)
                    .map_err(|e| {
                        CompileError::Codegen(format!("Failed to load variable {}: {:?}", name, e))
                    })?;
                // build_load returns a generic value; convert it to IntValue
                Ok(loaded_val.into_int_value())
            }
            Expr::Binary { op, left, right } => {
                // Compile left and right sub-expressions
                let left_val = self.compile_expr(left)?;
                let right_val = self.compile_expr(right)?;
                // Build the appropriate arithmetic instruction
                let result_val = match op {
                    BinOp::Add => self.builder.build_int_add(left_val, right_val, "addtmp"),
                    BinOp::Sub => self.builder.build_int_sub(left_val, right_val, "subtmp"),
                    BinOp::Mul => self.builder.build_int_mul(left_val, right_val, "multmp"),
                    BinOp::Div => {
                        // Use signed integer division
                        self.builder
                            .build_int_signed_div(left_val, right_val, "divtmp")
                    }
                };
                // unwrap or map any builder error (shouldn't fail if types are correct)
                let int_val = result_val.map_err(|e| {
                    CompileError::Codegen(format!("Failed to build {:?} operation: {:?}", op, e))
                })?;
                Ok(int_val)
            }
        }
    }
}
