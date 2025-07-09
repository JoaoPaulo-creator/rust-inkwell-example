(* open Ast
open Error
open Lexer
open Parser
open Codegen *)

let () =
  match Sys.argv with
  | [| _; path |] ->
      (try
        let src = In_channel.with_open_text path In_channel.input_all in
        let tokens = Lexer.lex src |> Result.get_ok in
        let parser = Parser.make_parser tokens in
        let prog = Parser.parse_program parser |> Result.get_ok in
        let llctx = Llvm.global_context () in
        let llmodule = Llvm.create_module llctx "toy" in
        let cg = Codegen.create_codegen llctx llmodule in
        Codegen.compile_program cg prog |> Result.get_ok;
        Llvm.print_module "program.ll" llmodule;
        (* JIT execution omitted for brevity *)
      with
      | Error.CompileError e -> Printf.eprintf "%s\n" (Error.string_of_error e); exit 1
      | Sys_error msg -> Printf.eprintf "IO error: %s\n" msg; exit 1)
  | _ -> Printf.eprintf "No input file specified\n"; exit 1
