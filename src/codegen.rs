use crate::ast::*;
use crate::error::CompileError;
use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, IntType},
    values::{FunctionValue, IntValue, PointerValue},
};
use std::collections::HashMap;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    i32_type: IntType<'ctx>,
    printf_fn: FunctionValue<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(ctx: &'ctx Context, module: Module<'ctx>) -> Self {
        let builder = ctx.create_builder();
        let i32_type = ctx.i32_type();

        // declare: i32 @printf(i8*, ...) external
        let i8_ptr = ctx.ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr.into()], true);
        let printf_fn = module.add_function("printf", printf_type, None);

        return CodeGen {
            context: ctx,
            module,
            builder,
            i32_type,
            printf_fn,
            variables: HashMap::new(),
        };
    }

    /// Entry‐point: compile all functions, then a `main` that runs global statements.
    pub fn compile_program(&mut self, prog: &Program) -> Result<(), CompileError> {
        // First, declare and define all user functions.
        for func in &prog.functions {
            self.compile_function_decl(func)?;
        }

        // Now generate `main`:
        let main_ty = self.i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_ty, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        // compile global statements
        for stmt in &prog.statements {
            self.compile_statement(stmt, Some(main_fn))?;
        }

        // default return 0 if none
        self.builder
            .build_return(Some(&self.i32_type.const_int(0, false)))?;
        Ok(())
    }

    /// Declare & define one user function.
    // codegen.rs
    fn compile_function_decl(&mut self, f: &Function) -> Result<(), CompileError> {
        // signature: i32 fn(i32, i32, …)
        let param_types = vec![self.i32_type.into(); f.params.len()];
        let fn_type = self.i32_type.fn_type(&param_types, false);
        let function = self.module.add_function(&f.name, fn_type, None);

        // entry block
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // allocate & store each parameter
        self.variables.clear();
        for (i, pname) in f.params.iter().enumerate() {
            let ptr = self.builder.build_alloca(self.i32_type, pname)?;
            self.builder
                .build_store(ptr, function.get_nth_param(i as u32).unwrap())?;
            self.variables.insert(pname.clone(), ptr);
        }

        // compile the body
        for stmt in &f.body {
            self.compile_statement(stmt, Some(function))?;
        }

        // if user forgot `return`, default to 0
        self.builder
            .build_return(Some(&self.i32_type.const_int(0, false)))?;
        Ok(())
    }

    /// Lower any statement.  `current_fn` is `Some(fn_val)` if we're inside a function (for `return`).
    fn compile_statement(
        &mut self,
        stmt: &Statement,
        current_fn: Option<FunctionValue<'ctx>>,
    ) -> Result<(), CompileError> {
        match stmt {
            Statement::VarDecl { name, expr } => {
                let val = self.compile_expr(expr)?;
                let ptr = self.builder.build_alloca(self.i32_type, name)?;
                self.builder.build_store(ptr, val)?;
                self.variables.insert(name.clone(), ptr);
            }
            Statement::Assign { name, expr } => {
                let val = self.compile_expr(expr)?;
                let ptr = if let Some(existing) = self.variables.get(name) {
                    *existing
                } else {
                    let new_ptr = self.builder.build_alloca(self.i32_type, name)?;
                    self.variables.insert(name.clone(), new_ptr);
                    new_ptr
                };

                let _ = self.builder.build_store(ptr, val)?;
            }
            Statement::Print { expr } => match expr {
                Expr::StrLiteral(s) => {
                    let fmt = self.builder.build_global_string_ptr("%s\n\0", "fmt")?;
                    let fmt_ptr = fmt.as_pointer_value();
                    let str_gv = self
                        .builder
                        .build_global_string_ptr(&format!("{}\0", s), "str");
                    let str_ptr = str_gv?.as_pointer_value();
                    let _ = self.builder.build_call(
                        self.printf_fn,
                        &[fmt_ptr.into(), str_ptr.into()],
                        "printstr",
                    );
                }

                _ => {
                    let val = self.compile_expr(expr)?;
                    let fmt = self.builder.build_global_string_ptr("%d\n\0", "fmt")?;
                    let fmt_ptr = fmt.as_pointer_value();
                    let _ = self.builder.build_call(
                        self.printf_fn,
                        &[fmt_ptr.into(), val.into()],
                        "printi",
                    );
                }
            },
            Statement::Return { expr } => {
                let val = self.compile_expr(expr)?;
                let _ = self.builder.build_return(Some(&val));
            }
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let test = self.compile_expr(cond)?;
                // zext i32->i1 for branching
                let zero = self.i32_type.const_int(0, false);
                let cond_i1 =
                    self.builder
                        .build_int_compare(IntPredicate::NE, test, zero, "ifcond")?;
                let parent = current_fn.unwrap();
                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let merge_bb = self.context.append_basic_block(parent, "ifcont");

                self.builder
                    .build_conditional_branch(cond_i1, then_bb, else_bb)?;

                // then
                self.builder.position_at_end(then_bb);
                for s in then_branch {
                    self.compile_statement(s, current_fn)?;
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    let _ = self.builder.build_unconditional_branch(merge_bb);
                }

                // else
                self.builder.position_at_end(else_bb);
                if let Some(els) = else_branch {
                    for s in els {
                        self.compile_statement(s, current_fn)?;
                    }
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    let _ = self.builder.build_unconditional_branch(merge_bb);
                }

                // continue
                self.builder.position_at_end(merge_bb);
            }
            Statement::While { cond, body } => {
                let parent = current_fn.unwrap();
                let loop_bb = self.context.append_basic_block(parent, "loop");
                let after_bb = self.context.append_basic_block(parent, "after");

                let _ = self.builder.build_unconditional_branch(loop_bb);
                let _ = self.builder.position_at_end(loop_bb);

                let test = self.compile_expr(cond)?;
                let zero = self.i32_type.const_int(0, false);
                let cond_i1 =
                    self.builder
                        .build_int_compare(IntPredicate::NE, test, zero, "whilecond")?;

                // body
                let body_bb = self.context.append_basic_block(parent, "body");
                self.builder
                    .build_conditional_branch(cond_i1, body_bb, after_bb)?;

                // build body
                self.builder.position_at_end(body_bb);
                for s in body {
                    self.compile_statement(s, current_fn)?;
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    let _ = self.builder.build_unconditional_branch(loop_bb);
                }

                // after
                let _ = self.builder.position_at_end(after_bb);
            }
            Statement::ExprStmt(e) => {
                let _ = self.compile_expr(e)?; // evaluate and discard
            }
        }
        Ok(())
    }

    /// Lower expressions to an `i32` `IntValue`.  Booleans come out as `i1` then `zext`→`i32`.
    fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, CompileError> {
        match expr {
            Expr::Number(n) => Ok(self.i32_type.const_int(*n as u64, true)),
            Expr::Bool(b) => {
                let i1 = self
                    .context
                    .bool_type()
                    .const_int(if *b { 1 } else { 0 }, false);
                let ext = self
                    .builder
                    .build_int_z_extend(i1, self.i32_type, "bool2int")?;
                Ok(ext)
            }
            Expr::StrLiteral(s) => {
                // create a global string pointer and return as i8*
                let gs = self
                    .builder
                    .build_global_string_ptr(&format!("{}\0", s), "strlit")?;
                let ptr_val = gs.as_pointer_value();
                let cast = self.builder.build_bit_cast(
                    ptr_val,
                    self.context
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum(),
                    "strtoint",
                )?;
                Ok(cast.into_int_value())
            }
            Expr::Variable(name) => {
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::Codegen(format!("undefined var {}", name)))?;
                let loaded = self.builder.build_load(self.i32_type, *ptr, name)?;
                Ok(loaded.into_int_value())
            }
            Expr::Unary { op, expr } => {
                let v = self.compile_expr(expr)?;
                match op {
                    UnOp::Pos => Ok(v),
                    UnOp::Neg => {
                        let neg = self.builder.build_int_sub(
                            self.i32_type.const_int(0, false),
                            v,
                            "negtmp",
                        )?;
                        Ok(neg)
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let rv = match op {
                    BinOp::Add => self.builder.build_int_add(l, r, "addtmp")?,
                    BinOp::Sub => self.builder.build_int_sub(l, r, "subtmp")?,
                    BinOp::Mul => self.builder.build_int_mul(l, r, "multmp")?,
                    BinOp::Div => self.builder.build_int_signed_div(l, r, "divtmp")?,
                    BinOp::Rem => self.builder.build_int_signed_rem(l, r, "remtmp")?,
                    BinOp::Lt => self.build_int_cmp(IntPredicate::SLT, l, r, "lttmp")?,
                    BinOp::Le => self.build_int_cmp(IntPredicate::SLE, l, r, "letmp")?,
                    BinOp::Gt => self.build_int_cmp(IntPredicate::SGT, l, r, "gttmp")?,
                    BinOp::Ge => self.build_int_cmp(IntPredicate::SGE, l, r, "getmp")?,
                    BinOp::Eq => self.build_int_cmp(IntPredicate::EQ, l, r, "eqtmp")?,
                    BinOp::Ne => self.build_int_cmp(IntPredicate::NE, l, r, "netmp")?,
                };
                Ok(rv)
            }
            Expr::Call { name, args } => {
                // lookup function
                let fn_val = self
                    .module
                    .get_function(name)
                    .ok_or_else(|| CompileError::Codegen(format!("unknown fn {}", name)))?;
                let mut compiled_args = Vec::new();
                for a in args {
                    compiled_args.push(self.compile_expr(a)?.into());
                }
                let call_site = self.builder.build_call(fn_val, &compiled_args, "calltmp")?;
                Ok(call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| CompileError::Codegen("call returned void".into()))?
                    .into_int_value())
            }
        }
    }

    /// helper for comparisons: build i1 then zext→i32
    fn build_int_cmp(
        &self,
        pred: IntPredicate,
        l: IntValue<'ctx>,
        r: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, CompileError> {
        let b = self.builder.build_int_compare(pred, l, r, name)?;
        let ext = self
            .builder
            .build_int_z_extend(b, self.i32_type, "bool2int")?;
        Ok(ext)
    }
}
