open Ast
open Error
open Lexer
open Parser
open Codegen

let () =
  match Sys.argv with
  | [| _; path |] ->
      (try
        let src = In_channel.with_open_text path In_channel.input_all in
        let tokens = lex src |> Result.get_ok in
        let parser = make_parser tokens in
        let prog = parse_program parser |> Result.get_ok in
        let llctx = Llvm.global_context () in
        let llmodule = Llvm.create_module llctx "toy" in
        let cg = create_codegen llctx llmodule in
        compile_program cg prog |> Result.get_ok;
        Llvm.print_module "program.ll" llmodule;
        (* JIT execution omitted for brevity *)
      with
      | CompileError e -> Printf.eprintf "%s\n" (string_of_error e); exit 1
      | Sys_error msg -> Printf.eprintf "IO error: %s\n" msg; exit 1)
  | _ -> Printf.eprintf "No input file specified\n"; exit 1
