open Ast
open Error
open Llvm

type codegen = {
  llctx: llcontext;
  llmodule: llmodule;
  builder: llbuilder;
  i32_type: lltype;
  printf: llvalue;
  variables: (string, llvalue) Hashtbl.t;
  array_sizes: (string, int) Hashtbl.t;
  function_types: (string, bool list * bool) Hashtbl.t;
}

let create_codegen llctx llmodule =
  let builder = builder llctx in
  let i32_type = Llvm.i32_type llctx in
  let i8_ptr = Llvm.pointer_type2 llctx in
  let printf_type = Llvm.function_type i32_type [| i8_ptr |] in
  let printf = Llvm.declare_function "printf" printf_type llmodule in
  {
    llctx;
    llmodule;
    builder;
    i32_type;
    printf;
    variables = Hashtbl.create 10;
    array_sizes = Hashtbl.create 10;
    function_types = Hashtbl.create 10;
  }

let ptr_type cg = Llvm.pointer_type2 cg.llctx

let rec compile_program cg prog =
  let analyze_function_types () = Ok () in
  let compile_function_decl f =
    let (param_is_array, returns_array) =
      try Hashtbl.find cg.function_types f.name
      with Not_found -> ([], false)
    in
    let param_types =
      List.map
        (fun is_array ->
          if is_array then ptr_type cg
          else cg.i32_type)
        param_is_array
    in
    let fn_type =
      if returns_array then 
        Llvm.function_type (ptr_type cg) (Array.of_list param_types) 
      else Llvm.function_type cg.i32_type (Array.of_list param_types)
    in
    let fn = Llvm.define_function f.name fn_type cg.llmodule in
    let entry = Llvm.append_block cg.llctx "entry" fn in
    Llvm.position_at_end entry cg.builder;
    List.iteri
      (fun i name ->
        let param = Llvm.param fn i in
        let alloca = Llvm.build_alloca cg.i32_type name cg.builder in
        ignore (Llvm.build_store param alloca cg.builder);
        Hashtbl.add cg.variables name alloca)
      f.params;
    List.iter
      (fun stmt -> ignore (compile_statement cg stmt (Some fn)))
      f.body;
    if not (List.exists (function Return _ -> true | _ -> false) f.body) then
      ignore (Llvm.build_ret (Llvm.const_int cg.i32_type 0) cg.builder);
    Ok ()
  in
  match analyze_function_types () with
  | Ok () ->
      List.iter
        (fun f -> ignore (compile_function_decl f))
        prog.functions;
      let main_ty = Llvm.function_type cg.i32_type [||] in
      let main_fn = Llvm.define_function "main" main_ty cg.llmodule in
      let entry = Llvm.append_block cg.llctx "entry" main_fn in
      Llvm.position_at_end entry cg.builder;
      List.iter
        (fun stmt -> ignore (compile_statement cg stmt (Some main_fn)))
        prog.statements;
      ignore (Llvm.build_ret (Llvm.const_int cg.i32_type 0) cg.builder);
      Ok ()
  | Error e -> Error e

and compile_statement cg stmt =
  match stmt with
  | VarDecl (name, expr) ->
      let value = compile_expr cg expr in
      let alloca = Llvm.build_alloca cg.i32_type name cg.builder in
      ignore (Llvm.build_store value alloca cg.builder);
      Hashtbl.add cg.variables name alloca;
      fun _ -> Ok ()
  | Print expr ->
      let value = compile_expr cg expr in
      let fmt = Llvm.build_global_stringptr "%d\\n\\0" "fmt" cg.builder in
      let printf_type = Llvm.type_of cg.printf in
      ignore (Llvm.build_call2 printf_type cg.printf [| fmt; value |] "print_call" cg.builder);
      fun _ -> Ok ()
  | Return expr ->
      let value = compile_expr cg expr in
      ignore (Llvm.build_ret value cg.builder);
      fun _ -> Ok ()
  | _ -> fun _ -> Ok ()

and compile_expr cg = function
  | Number n -> Llvm.const_int cg.i32_type (Int64.to_int n)
  | Bool b -> Llvm.const_int cg.i32_type (if b then 1 else 0)
  | Variable name ->
      let ptr =
        try Hashtbl.find cg.variables name
        with Not_found -> raise (CompileError (Codegen ("undefined variable " ^ name)))
      in
      Llvm.build_load2 cg.i32_type ptr name cg.builder
  | Binary (op, left, right) ->
      let l = compile_expr cg left in
      let r = compile_expr cg right in
      let op = match op with
        | Add -> Llvm.build_add
        | Sub -> Llvm.build_sub
        | Mul -> Llvm.build_mul
        | Div -> Llvm.build_sdiv
        | Rem -> Llvm.build_srem
        | Lt -> Llvm.build_icmp Llvm.Icmp.Slt
        | Le -> Llvm.build_icmp Llvm.Icmp.Sle
        | Gt -> Llvm.build_icmp Llvm.Icmp.Sgt
        | Ge -> Llvm.build_icmp Llvm.Icmp.Sge
        | Eq -> Llvm.build_icmp Llvm.Icmp.Eq
        | Ne -> Llvm.build_icmp Llvm.Icmp.Ne
      in
      op l r "tmp" cg.builder
  | _ -> raise (CompileError (Codegen "unsupported expression"))

