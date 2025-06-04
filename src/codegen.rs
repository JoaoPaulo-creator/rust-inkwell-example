use crate::ast::*;
use crate::error::CompileError;
use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    types::IntType,
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
    array_sizes: HashMap<String, usize>,
    function_types: HashMap<String, (Vec<bool>, bool)>, // (param_is_array, returns_array)
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(ctx: &'ctx Context, module: Module<'ctx>) -> Self {
        let builder = ctx.create_builder();
        let i32_type = ctx.i32_type();

        let i8_ptr = ctx.ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr.into()], true);
        let printf_fn = module.add_function("printf", printf_type, None);

        CodeGen {
            context: ctx,
            module,
            builder,
            i32_type,
            printf_fn,
            variables: HashMap::new(),
            array_sizes: HashMap::new(),
            function_types: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, prog: &Program) -> Result<(), CompileError> {
        self.analyze_function_types(prog)?;

        for func in &prog.functions {
            self.compile_function_decl(func)?;
        }

        let main_ty = self.i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_ty, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &prog.statements {
            self.compile_statement(stmt, Some(main_fn))?;
        }

        self.builder
            .build_return(Some(&self.i32_type.const_int(0, false)))?;
        Ok(())
    }

    fn analyze_function_types(&mut self, prog: &Program) -> Result<(), CompileError> {
        for func in &prog.functions {
            let mut param_is_array = vec![false; func.params.len()];
            let mut returns_array = false;

            // Analyze function body for return type
            for stmt in &func.body {
                if let Statement::Return { expr } = stmt {
                    if matches!(expr, Expr::ArrayLiteral(_)) {
                        returns_array = true;
                    }
                }
            }

            // Analyze calls to this function for parameter types
            for other_func in &prog.functions {
                for stmt in &other_func.body {
                    self.analyze_stmt_for_calls(&func.name, &mut param_is_array, stmt, prog)?;
                }
            }
            for stmt in &prog.statements {
                self.analyze_stmt_for_calls(&func.name, &mut param_is_array, stmt, prog)?;
            }

            self.function_types
                .insert(func.name.clone(), (param_is_array, returns_array));
        }
        Ok(())
    }

    fn analyze_stmt_for_calls(
        &self,
        func_name: &str,
        param_is_array: &mut [bool],
        stmt: &Statement,
        prog: &Program,
    ) -> Result<(), CompileError> {
        match stmt {
            Statement::ExprStmt(expr) | Statement::Return { expr } | Statement::Print { expr } => {
                self.analyze_expr_for_calls(func_name, param_is_array, expr, prog)?;
            }
            Statement::VarDecl { expr, .. }
            | Statement::LetDecl { expr, .. }
            | Statement::Assign { expr, .. } => {
                self.analyze_expr_for_calls(func_name, param_is_array, expr, prog)?;
            }
            Statement::IndexedAssign {
                array, index, expr, ..
            } => {
                self.analyze_expr_for_calls(func_name, param_is_array, array, prog)?;
                self.analyze_expr_for_calls(func_name, param_is_array, index, prog)?;
                self.analyze_expr_for_calls(func_name, param_is_array, expr, prog)?;
            }
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.analyze_expr_for_calls(func_name, param_is_array, cond, prog)?;
                for s in then_branch {
                    self.analyze_stmt_for_calls(func_name, param_is_array, s, prog)?;
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        self.analyze_stmt_for_calls(func_name, param_is_array, s, prog)?;
                    }
                }
            }
            Statement::While { cond, body } => {
                self.analyze_expr_for_calls(func_name, param_is_array, cond, prog)?;
                for s in body {
                    self.analyze_stmt_for_calls(func_name, param_is_array, s, prog)?;
                }
            }
        }
        Ok(())
    }

    fn analyze_expr_for_calls(
        &self,
        func_name: &str,
        param_is_array: &mut [bool],
        expr: &Expr,
        prog: &Program,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::Call { name, args } if name == func_name => {
                for (i, arg) in args.iter().enumerate() {
                    if i >= param_is_array.len() {
                        return Err(CompileError::Codegen(format!(
                            "Too many arguments for function {}",
                            func_name
                        )));
                    }
                    if matches!(arg, Expr::Variable(var) if self.array_sizes.contains_key(var))
                        || matches!(arg, Expr::ArrayLiteral(_))
                    {
                        param_is_array[i] = true;
                    }
                }
            }
            Expr::Length { array } => {
                if let Expr::Variable(var) = &**array {
                    // Check if the variable is a parameter of the function
                    if let Some(func) = prog.functions.iter().find(|f| f.name == func_name) {
                        if let Some(idx) = func.params.iter().position(|p| p == var) {
                            param_is_array[idx] = true;
                        }
                    }
                    self.analyze_expr_for_calls(func_name, param_is_array, array, prog)?;
                }
            }
            Expr::Unary { expr, .. } => {
                self.analyze_expr_for_calls(func_name, param_is_array, expr, prog)?;
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expr_for_calls(func_name, param_is_array, left, prog)?;
                self.analyze_expr_for_calls(func_name, param_is_array, right, prog)?;
            }
            Expr::Index { array, index } => {
                self.analyze_expr_for_calls(func_name, param_is_array, array, prog)?;
                self.analyze_expr_for_calls(func_name, param_is_array, index, prog)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_function_decl(&mut self, f: &Function) -> Result<(), CompileError> {
        let (param_is_array, returns_array) = self.function_types.get(&f.name).unwrap().clone();
        let mut param_types = Vec::new();
        for is_array in &param_is_array {
            if *is_array {
                param_types.push(self.context.ptr_type(AddressSpace::default()).into()); // Array pointer
                param_types.push(self.i32_type.into()); // Array size
            } else {
                param_types.push(self.i32_type.into()); // Scalar
            }
        }

        let fn_type = if returns_array {
            self.context
                .ptr_type(AddressSpace::default())
                .fn_type(&param_types, false)
        } else {
            self.i32_type.fn_type(&param_types, false)
        };
        let function = self.module.add_function(&f.name, fn_type, None);

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.array_sizes.clear();
        let mut param_idx = 0;
        for (i, pname) in f.params.iter().enumerate() {
            if param_is_array[i] {
                let ptr = function.get_nth_param(param_idx).ok_or_else(|| {
                    CompileError::Codegen(format!("missing array pointer param for {}", pname))
                })?;
                let size = function.get_nth_param(param_idx + 1).ok_or_else(|| {
                    CompileError::Codegen(format!("missing array size param for {}", pname))
                })?;
                let size_val = size
                    .into_int_value()
                    .get_sign_extended_constant()
                    .ok_or_else(|| {
                        CompileError::Codegen(format!("array size for {} must be constant", pname))
                    })?;
                let alloca = self
                    .builder
                    .build_alloca(self.context.ptr_type(AddressSpace::default()), pname)?;
                self.builder.build_store(alloca, ptr)?;
                self.variables.insert(pname.clone(), alloca);
                self.array_sizes.insert(pname.clone(), size_val as usize);
                param_idx += 2;
            } else {
                let ptr = self.builder.build_alloca(self.i32_type, pname)?;
                self.builder
                    .build_store(ptr, function.get_nth_param(param_idx).unwrap())?;
                self.variables.insert(pname.clone(), ptr);
                param_idx += 1;
            }
        }

        for stmt in &f.body {
            self.compile_statement(stmt, Some(function))?;
        }

        if returns_array {
            let empty_array = self.i32_type.array_type(0);
            let alloca = self.builder.build_alloca(empty_array, "empty_array")?;
            self.builder.build_return(Some(&alloca))?;
        } else {
            self.builder
                .build_return(Some(&self.i32_type.const_int(0, false)))?;
        }
        Ok(())
    }

    fn compile_statement(
        &mut self,
        stmt: &Statement,
        current_fn: Option<FunctionValue<'ctx>>,
    ) -> Result<(), CompileError> {
        match stmt {
            Statement::VarDecl { name, expr } => {
                let ptr = if let Expr::ArrayLiteral(elems) = expr {
                    self.compile_array_literal(elems, name)?
                } else {
                    let val = self.compile_expr(expr)?;
                    let ptr = self.builder.build_alloca(self.i32_type, name)?;
                    self.builder.build_store(ptr, val)?;
                    ptr
                };
                self.variables.insert(name.clone(), ptr);
            }
            Statement::LetDecl { name, expr } => {
                let ptr = if let Expr::ArrayLiteral(elems) = expr {
                    let array_ptr = self.compile_array_literal(elems, name)?;
                    let ptr = self
                        .builder
                        .build_alloca(self.context.ptr_type(AddressSpace::default()), name)?;
                    self.builder.build_store(ptr, array_ptr)?;
                    self.array_sizes.insert(name.clone(), elems.len());
                    ptr
                } else if let Expr::Variable(var_name) = expr {
                    if self.array_sizes.contains_key(var_name) {
                        let array_ptr = self.load_array_ptr(var_name)?;
                        let ptr = self
                            .builder
                            .build_alloca(self.context.ptr_type(AddressSpace::default()), name)?;
                        self.builder.build_store(ptr, array_ptr)?;
                        let size = *self.array_sizes.get(var_name).ok_or_else(|| {
                            CompileError::Codegen(format!("Array size not found for {}", var_name))
                        })?;
                        self.array_sizes.insert(name.clone(), size);
                        ptr
                    } else {
                        let val = self.compile_expr(expr)?;
                        let ptr = self.builder.build_alloca(self.i32_type, name)?;
                        self.builder.build_store(ptr, val)?;
                        ptr
                    }
                } else {
                    let val = self.compile_expr(expr)?;
                    let ptr = self.builder.build_alloca(self.i32_type, name)?;
                    self.builder.build_store(ptr, val)?;
                    ptr
                };
                self.variables.insert(name.clone(), ptr);
            }
            Statement::Assign { name, expr } => {
                let ptr = *self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::Codegen(format!("undefined variable {}", name)))?;
                if let Expr::ArrayLiteral(elems) = expr {
                    let new_ptr = self.compile_array_literal(elems, name)?;
                    self.variables.insert(name.clone(), new_ptr);
                    self.array_sizes.insert(name.clone(), elems.len());
                } else if let Expr::Call { name: fn_name, .. } = expr {
                    let returns_array = self
                        .function_types
                        .get(fn_name)
                        .map(|(_, ret)| *ret)
                        .unwrap_or(false);
                    if returns_array {
                        let val = self.compile_expr(expr)?;
                        let ptr_val = self.builder.build_int_to_ptr(
                            val,
                            self.context.ptr_type(AddressSpace::default()),
                            "int_to_ptr",
                        )?;
                        let alloca = self
                            .builder
                            .build_alloca(self.context.ptr_type(AddressSpace::default()), name)?;
                        self.builder.build_store(alloca, ptr_val)?;
                        self.variables.insert(name.clone(), alloca);
                        self.array_sizes
                            .insert(name.clone(), self.get_array_size_from_context(name)?);
                    } else {
                        let val = self.compile_expr(expr)?;
                        self.builder.build_store(ptr, val)?;
                    }
                } else {
                    let val = self.compile_expr(expr)?;
                    self.builder.build_store(ptr, val)?;
                }
            }
            Statement::IndexedAssign { array, index, expr } => {
                let array_name = match &**array {
                    Expr::Variable(name) => name,
                    _ => {
                        return Err(CompileError::Codegen(
                            "Array in indexed assignment must be a variable".into(),
                        ));
                    }
                };
                let array_ptr = self.load_array_ptr(array_name)?;
                let idx = self.compile_expr(index)?;
                let val = self.compile_expr(expr)?;
                let array_type = self.i32_type.array_type(0);
                let ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        array_type,
                        array_ptr,
                        &[self.i32_type.const_int(0, false), idx],
                        "index_ptr",
                    )?
                };
                self.builder.build_store(ptr, val)?;
            }
            Statement::Print { expr } => match expr {
                Expr::StrLiteral(s) => {
                    let fmt = self.builder.build_global_string_ptr("%s\n\0", "fmt")?;
                    let str_gv = self
                        .builder
                        .build_global_string_ptr(&format!("{}\0", s), "str")?;
                    self.builder.build_call(
                        self.printf_fn,
                        &[
                            fmt.as_pointer_value().into(),
                            str_gv.as_pointer_value().into(),
                        ],
                        "print_call",
                    )?;
                }
                _ => {
                    let val = self.compile_expr(expr)?;
                    let fmt = self.builder.build_global_string_ptr("%d\n\0", "fmt")?;
                    self.builder.build_call(
                        self.printf_fn,
                        &[fmt.as_pointer_value().into(), val.into()],
                        "print_call",
                    )?;
                }
            },
            Statement::Return { expr } => {
                if let Expr::ArrayLiteral(elems) = expr {
                    let array_ptr = self.compile_array_literal(elems, "ret_array")?;
                    self.builder.build_return(Some(&array_ptr))?;
                } else {
                    let val = self.compile_expr(expr)?;
                    self.builder.build_return(Some(&val))?;
                }
            }
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let test = self.compile_expr(cond)?;
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
                    self.builder.build_unconditional_branch(merge_bb)?;
                }

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
                    self.builder.build_unconditional_branch(merge_bb)?;
                }

                self.builder.position_at_end(merge_bb);
            }
            Statement::While { cond, body } => {
                let parent = current_fn.unwrap();
                let loop_bb = self.context.append_basic_block(parent, "loop");
                let after_bb = self.context.append_basic_block(parent, "after");

                self.builder.build_unconditional_branch(loop_bb)?;
                self.builder.position_at_end(loop_bb);

                let test = self.compile_expr(cond)?;
                let zero = self.i32_type.const_int(0, false);
                let cond_i1 =
                    self.builder
                        .build_int_compare(IntPredicate::NE, test, zero, "whilecond")?;

                let body_bb = self.context.append_basic_block(parent, "body");
                self.builder
                    .build_conditional_branch(cond_i1, body_bb, after_bb)?;

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
                    self.builder.build_unconditional_branch(loop_bb)?;
                }

                self.builder.position_at_end(after_bb);
            }
            Statement::ExprStmt(e) => {
                self.compile_expr(e)?;
            }
        }
        Ok(())
    }

    fn compile_array_literal(
        &mut self,
        elems: &[Expr],
        name: &str,
    ) -> Result<PointerValue<'ctx>, CompileError> {
        let array_type = self.i32_type.array_type(elems.len() as u32);
        let alloca = self.builder.build_alloca(array_type, name)?;
        for (i, elem) in elems.iter().enumerate() {
            let val = self.compile_expr(elem)?;
            let ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    array_type,
                    alloca,
                    &[
                        self.i32_type.const_int(0, false),
                        self.i32_type.const_int(i as u64, false),
                    ],
                    "elem_ptr",
                )?
            };
            self.builder.build_store(ptr, val)?;
        }
        self.array_sizes.insert(name.to_string(), elems.len());
        Ok(alloca)
    }

    fn load_array_ptr(&mut self, array_name: &str) -> Result<PointerValue<'ctx>, CompileError> {
        let ptr = self
            .variables
            .get(array_name)
            .ok_or_else(|| CompileError::Codegen(format!("undefined array {}", array_name)))?;
        if self.array_sizes.contains_key(array_name) {
            let loaded = self.builder.build_load(
                self.context.ptr_type(AddressSpace::default()),
                *ptr,
                "load_array_ptr",
            )?;
            Ok(loaded.into_pointer_value())
        } else {
            Ok(*ptr)
        }
    }

    fn get_array_size_from_context(&self, name: &str) -> Result<usize, CompileError> {
        self.array_sizes
            .get(name)
            .copied()
            .ok_or_else(|| CompileError::Codegen(format!("Array size not found for {}", name)))
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, CompileError> {
        match expr {
            Expr::Number(n) => Ok(self.i32_type.const_int(*n as u64, true)),
            Expr::Bool(b) => {
                let i1 = self
                    .context
                    .bool_type()
                    .const_int(if *b { 1 } else { 0 }, false);
                Ok(self
                    .builder
                    .build_int_z_extend(i1, self.i32_type, "bool2int")?)
            }
            Expr::StrLiteral(s) => {
                let gs = self
                    .builder
                    .build_global_string_ptr(&format!("{}\0", s), "strlit")?;
                let ptr_val = gs.as_pointer_value();
                let cast = self.builder.build_bit_cast(
                    ptr_val,
                    self.context.ptr_type(AddressSpace::default()),
                    "strtoint",
                )?;
                Ok(cast.into_int_value())
            }
            Expr::Variable(name) => {
                if self.array_sizes.contains_key(name) {
                    let ptr = self.load_array_ptr(name)?;
                    let cast = self
                        .builder
                        .build_ptr_to_int(ptr, self.i32_type, "array_to_i32")?;
                    Ok(cast)
                } else {
                    let ptr = self
                        .variables
                        .get(name)
                        .ok_or_else(|| CompileError::Codegen(format!("undefined var {}", name)))?;
                    let loaded = self.builder.build_load(self.i32_type, *ptr, name)?;
                    Ok(loaded.into_int_value())
                }
            }
            Expr::Unary { op, expr } => {
                let v = self.compile_expr(expr)?;
                match op {
                    UnOp::Pos => Ok(v),
                    UnOp::Neg => Ok(self.builder.build_int_sub(
                        self.i32_type.const_int(0, false),
                        v,
                        "negtmp",
                    )?),
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
                let fn_val = self
                    .module
                    .get_function(name)
                    .ok_or_else(|| CompileError::Codegen(format!("unknown fn {}", name)))?;
                let (param_is_array, returns_array) =
                    self.function_types.get(name).unwrap().clone();
                let mut compiled_args = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    if i < param_is_array.len() && param_is_array[i] {
                        if let Expr::Variable(var_name) = arg {
                            if self.array_sizes.contains_key(var_name) {
                                let ptr = self.load_array_ptr(var_name)?;
                                let size = *self.array_sizes.get(var_name).unwrap();
                                compiled_args.push(ptr.into());
                                compiled_args
                                    .push(self.i32_type.const_int(size as u64, false).into());
                                continue;
                            }
                        } else if let Expr::ArrayLiteral(elems) = arg {
                            let ptr = self.compile_array_literal(elems, "arg_array")?;
                            compiled_args.push(ptr.into());
                            compiled_args
                                .push(self.i32_type.const_int(elems.len() as u64, false).into());
                            continue;
                        }
                        return Err(CompileError::Codegen(format!(
                            "Expected array argument for parameter {} of {}",
                            i, name
                        )));
                    }
                    let val = self.compile_expr(arg)?;
                    compiled_args.push(val.into());
                }
                let call_site = self.builder.build_call(fn_val, &compiled_args, "calltmp")?;
                if returns_array {
                    let ptr = call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| CompileError::Codegen("array return expected".into()))?
                        .into_pointer_value();
                    let cast = self
                        .builder
                        .build_ptr_to_int(ptr, self.i32_type, "array_to_i32")?;
                    Ok(cast)
                } else {
                    Ok(call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| CompileError::Codegen("scalar return expected".into()))?
                        .into_int_value())
                }
            }
            Expr::ArrayLiteral(elems) => {
                let array_ptr = self.compile_array_literal(elems, "array")?;
                let cast =
                    self.builder
                        .build_ptr_to_int(array_ptr, self.i32_type, "array_to_i32")?;
                Ok(cast)
            }
            Expr::Index { array, index } => {
                let array_name = match &**array {
                    Expr::Variable(name) => name,
                    _ => {
                        return Err(CompileError::Codegen(
                            "Array indexing must use a variable".into(),
                        ));
                    }
                };
                let array_ptr = self.load_array_ptr(array_name)?;
                let idx = self.compile_expr(index)?;
                let size = *self.array_sizes.get(array_name).ok_or_else(|| {
                    CompileError::Codegen(format!("undefined array {}", array_name))
                })?;
                let idx_val = idx.get_sign_extended_constant().ok_or_else(|| {
                    CompileError::Codegen("Index must be a constant or resolvable integer".into())
                })?;
                if idx_val < 0 || idx_val as usize >= size {
                    return Err(CompileError::Codegen(format!(
                        "Index {} out of bounds for array {} of size {}",
                        idx_val, array_name, size
                    )));
                }
                let array_type = self.i32_type.array_type(0);
                let ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        array_type,
                        array_ptr,
                        &[self.i32_type.const_int(0, false), idx],
                        "index_ptr",
                    )?
                };
                let loaded = self.builder.build_load(self.i32_type, ptr, "index_load")?;
                Ok(loaded.into_int_value())
            }
            Expr::Length { array } => {
                let array_name = match &**array {
                    Expr::Variable(name) => name,
                    _ => {
                        return Err(CompileError::Codegen(
                            "Length must be called on a variable".into(),
                        ));
                    }
                };
                let size = *self.array_sizes.get(array_name).ok_or_else(|| {
                    CompileError::Codegen(format!("undefined array {}", array_name))
                })?;
                Ok(self.i32_type.const_int(size as u64, false))
            }
        }
    }

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

impl Expr {
    fn array_len(&self) -> Option<usize> {
        match self {
            Expr::ArrayLiteral(elems) => Some(elems.len()),
            _ => None,
        }
    }
}
