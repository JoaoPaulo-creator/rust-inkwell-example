open Printf
open Sys

(* Read an entire file into a string *)
let read_file filename =
  let ic = open_in filename in
  let buf = Buffer.create 4096 in
  (try
     while true do
       Buffer.add_string buf (input_line ic ^ "\n")
     done
   with End_of_file -> ());
  close_in ic;
  Buffer.contents buf

let () =
  (* Ensure a source file was provided *)
  if Array.length argv < 2 then begin
    eprintf "Usage: %s <source.toy>\n" argv.(0);
    exit 1
  end;

  let filename = argv.(1) in
  let src      = read_file filename in

  (* === Lexing === *)
  let tokens = Lexer.lex src in
  List.iter (fun t -> eprintf "%s " (Lexer.string_of_token t)) tokens;
  eprintf "\n%!";  (* dump token stream *)

  (* === Parsing === *)
  let parser = Parser.make_parser tokens in
  let prog =
    match Parser.parse_program parser with
    | Error e -> failwith ("parse error: " ^ Error.string_of_error e)
    | Ok p    -> p
  in
  eprintf ">>> parsing yielded %d top-level statements\n%!" (List.length prog.statements);

  (* === Codegen === *)
  let llctx = Llvm.global_context () in
  let llmod = Llvm.create_module llctx "toy" in
  let cg = Codegen.create_codegen llctx llmod in
  (match Codegen.compile_program cg prog with
  | Error e -> failwith ("codegen error: " ^ Error.string_of_error e)
  | Ok ()   -> ()
);

  (* === Dump IR === *)
  let ir_file = "program.ll" in
  Llvm.print_module ir_file cg.llmodule;
  eprintf ">> dumping IR at %s/%s\n%!" (Sys.getcwd ()) ir_file;