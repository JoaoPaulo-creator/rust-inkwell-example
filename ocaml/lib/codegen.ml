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

(* Create a global string constant and return a pointer to its data *)
let create_global_string cg str name =
  let global = Llvm.define_global name (Llvm.const_string cg.llctx str) cg.llmodule in
  let str_type = Llvm.array_type (Llvm.i8_type cg.llctx) (String.length str + 1) in
  Llvm.set_initializer (Llvm.const_string cg.llctx str) global;
  Llvm.const_gep2 str_type global [| Llvm.const_int cg.i32_type 0; Llvm.const_int cg.i32_type 0 |]

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
    
    (* Handle function parameters *)
    List.iteri
      (fun i name ->
        let param = Llvm.param fn i in
        let alloca = Llvm.build_alloca cg.i32_type name cg.builder in
        ignore (Llvm.build_store param alloca cg.builder);
        Hashtbl.add cg.variables name alloca)
      f.params;
    
    (* Compile function body *)
    List.iter (fun stmt -> 
      match compile_statement cg stmt (Some fn) with
      | Ok () -> ()
      | Error e -> raise (CompileError e)
    ) f.body;
    
    (* Add return if missing *)
    if not (List.exists (function Return _ -> true | _ -> false) f.body) then
      ignore (Llvm.build_ret (Llvm.const_int cg.i32_type 0) cg.builder);
    
    Ok ()
  in
  
  match analyze_function_types () with
  | Ok () ->
      (* Compile all functions *)
      List.iter (fun f -> 
        match compile_function_decl f with
        | Ok () -> ()
        | Error e -> raise (CompileError e)
      ) prog.functions;
      
      (* Compile main function *)
      let main_ty = Llvm.function_type cg.i32_type [||] in
      let main_fn = Llvm.define_function "main" main_ty cg.llmodule in
      let entry = Llvm.append_block cg.llctx "entry" main_fn in
      Llvm.position_at_end entry cg.builder;
      
      (* Compile program statements *)
      List.iter (fun stmt -> 
        match compile_statement cg stmt (Some main_fn) with
        | Ok () -> ()
        | Error e -> raise (CompileError e)
      ) prog.statements;
      
      (* Add final return *)
      if not (List.exists (function Return _ -> true | _ -> false) prog.statements) then
        ignore (Llvm.build_ret (Llvm.const_int cg.i32_type 0) cg.builder);
      
      Ok ()
  | Error e -> Error e

and compile_statement cg stmt current_fn =
  match stmt with
  | VarDecl (name, expr) ->
      let value = compile_expr cg expr in
      let alloca = Llvm.build_alloca cg.i32_type name cg.builder in
      ignore (Llvm.build_store value alloca cg.builder);
      Hashtbl.add cg.variables name alloca;
      Ok ()
  
  | Print expr ->
      let value = compile_expr cg expr in
      let (fmt_str, name) = 
        if Llvm.type_of value = cg.i32_type then 
          ("%d\\n\\0", "fmt_int") 
        else 
          ("%s\\n\\0", "fmt_str")
      in
      let fmt = create_global_string cg fmt_str name in
      let printf_type = Llvm.type_of cg.printf in
      ignore (Llvm.build_call2 printf_type cg.printf [| fmt; value |] "print_call" cg.builder);
      Ok ()
  
  | Return expr ->
      let value = compile_expr cg expr in
      ignore (Llvm.build_ret value cg.builder);
      Ok ()
  
  | While (cond, body) -> 
      let fn = match current_fn with
        | Some f -> f
        | None -> failwith "While loop not in function context"
      in
      let cond_bb = Llvm.append_block cg.llctx "while.cond" fn in
      let body_bb = Llvm.append_block cg.llctx "while.body" fn in
      let end_bb = Llvm.append_block cg.llctx "while.end" fn in
      
      (* Jump to condition block *)
      ignore (Llvm.build_br cond_bb cg.builder);
      
      (* Condition block *)
      Llvm.position_at_end cond_bb cg.builder;
      let cond_val = compile_expr cg cond in
      let zero = Llvm.const_int cg.i32_type 0 in
      let cmp = Llvm.build_icmp Llvm.Icmp.Ne cond_val zero "whilecond" cg.builder in
      ignore (Llvm.build_cond_br cmp body_bb end_bb cg.builder);
      
      (* Body block *)
      Llvm.position_at_end body_bb cg.builder;
      List.iter (fun stmt -> 
        match compile_statement cg stmt current_fn with
        | Ok () -> ()
        | Error e -> raise (CompileError e)
      ) body;
      ignore (Llvm.build_br cond_bb cg.builder);
      
      (* Position builder at end block *)
      Llvm.position_at_end end_bb cg.builder;
      Ok ()
  
  | Assign (name, expr) ->
      let value = compile_expr cg expr in
      let ptr =
        try Hashtbl.find cg.variables name
        with Not_found -> raise (CompileError (Codegen ("undefined variable " ^ name)))
      in
      ignore (Llvm.build_store value ptr cg.builder);
      Ok ()
  
  | _ -> Ok ()

and compile_expr cg = function 
  | StrLiteral s -> 
    let global = Llvm.define_global "str" (Llvm.const_string cg.llctx s) cg.llmodule in 
    let str_type = Llvm.array_type (Llvm.i8_type cg.llctx) (String.length s + 1) in 
    Llvm.set_initializer (Llvm.const_string cg.llctx s) global;
    Llvm.const_gep2 str_type global [| Llvm.const_int cg.i32_type 0; Llvm.const_int cg.i32_type 0 |]

  | Number n -> Llvm.const_int cg.i32_type (Int64.to_int n)

  | Bool b -> Llvm.const_int cg.i32_type (if b then 1 else 0)

  | Variable name -> 
    let ptr = 
      try Hashtbl.find cg.variables name 
      with Not_found -> raise (CompileError (Codegen ("undefined varialbe " ^ name))) 
    in 
    Llvm.build_load2 cg.i32_type ptr "variable" cg.builder

  | Unary (op, expr) ->
    let e = compile_expr cg expr in 
    let res =
      match op with 
      | Pos -> e
      | Neg -> Llvm.build_neg e "negtmp" cg.builder
    in
    res

  | Call (name, args) ->
    let func =
      match Llvm.lookup_function name cg.llmodule with
      | Some f -> f 
      | None -> raise (CompileError (Codegen ("undefined function " ^ name)))
    in
    let compile_args = List.map (compile_expr cg) args 
    in 
    let func_type = Llvm.type_of func 
    in 
    Llvm.build_call2 func_type func (Array.of_list compile_args) "calltmp" cg.builder

  | ArrayLiteral elems ->
    let array_type = Llvm.array_type cg.i32_type (List.length elems) in 
    let alloca = Llvm.build_alloca array_type "array" cg.builder in 
    List.iteri (fun i elem ->
      let elem_ptr = 
        Llvm.build_gep2 array_type
        alloca 
        [|Llvm.const_int cg.i32_type 0; Llvm.const_int cg.i32_type i|]
        "elem_ptr" cg.builder
      in
      let val_ = compile_expr cg elem in 
      ignore (Llvm.build_store val_ elem_ptr cg.builder)
      ) elems;
      alloca 

  | Index (array, index) ->
    let array_ptr = compile_expr cg array in 
    let index_val = compile_expr cg index in 
    let array_type =
      Llvm.element_type (Llvm.type_of array_ptr)
    in
    let elem_ptr =
      Llvm.build_gep2
        array_type
        array_ptr
        [| Llvm.const_int cg.i32_type 0; index_val |]
        "elem_ptr"
        cg.builder
    in
    Llvm.build_load2  
      cg.i32_type
      elem_ptr
      "elem"
      cg.builder
  | Length array ->
    let array_ptr = compile_expr cg array in
    let array_type = Llvm.type_of array_ptr in 
    let res = 
      match array_type with
      | Llvm.TypeKind.Array -> 
        Llvm.const_int cg.i32_type (Llvm.array_length array_type)
      | _ -> raise (CompileError (Codegen "length operator can only be used on array"))
    in res
  | Binary (op, left, right) ->
    let l = compile_expr cg left in 
    let r = compile_expr cg right in
    let res = match op with 
      | Add -> Llvm.build_add l r "addtmp" cg.builder
      | Sub -> Llvm.build_sub l r "subtmp" cg.builder 
      | Mul -> Llvm.build_mul l r "multmp" cg.builder
      | Div -> Llvm.build_sdiv l r "divtmp" cg.builder 
      | Rem -> Llvm.build_srem l r "remtmp" cg.builder
      | Lt -> Llvm.build_icmp Llvm.Icmp.Slt l r "lttmp" cg.builder
      | Le -> Llvm.build_icmp Llvm.Icmp.Sle l r "letmp" cg.builder
      | Gt -> Llvm.build_icmp Llvm.Icmp.Sgt l r "gttmp" cg.builder
      | Ge -> Llvm.build_icmp Llvm.Icmp.Sge l r "getmp" cg.builder
      | Eq -> Llvm.build_icmp Llvm.Icmp.Eq l r "eqtmp" cg.builder
      | Ne -> Llvm.build_icmp Llvm.Icmp.Ne l r "netmp" cg.builder
    in 
    res
  | _ -> raise (CompileError (Codegen "unsupported expression"))