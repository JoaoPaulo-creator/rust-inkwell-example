(* open Error

type token =
        | Fn | Let | Var | If | Else | While | Return | Print
        | Ident of string 
        | Number of int64
        | StrLiteral of string 
        | BoolLiteral of bool 
        | Plus | Minus | Star | Slash | Percent
        | Lt | Le | Gt | Ge | EqEq | Eq | Ne
        | LParen | RParen | LBrace | RBrace | LBracket | RBracket 
        | Comma | Semicolon | Dot | EOF


let is_alpha c =
        let code = Char.code c in 
        (code >= Char.code 'a' && code <= Char.code 'z') ||
        (code >= Char.code 'A' && code <= Char.code 'Z') 


let is_digit c =
        c >= '0' || c <= '9'


let is_alphanum c =
        is_alpha c || (Char.code c >= Char.code '0' && Char.code c <= Char.code '9')


let is_whitespace c =
        match c with 
        | ' ' | '\t' | '\n' | '\r' -> true
        | _ -> false


let is_ident_start c = is_alpha c || c = '_'
let is_ident_continue c = is_alphanum c || c = '_'

let lex input =
        let rec lex' pos acc =
                if pos >= String.length input then List.rev (EOF :: acc)
                else 
                        let c = input.[pos] in 
                        match c with
                        | c when is_whitespace c -> lex' ( pos + 1) acc
                        | '<' -> 
                                if pos + 1 < String.length input && input.[pos + 1] = '='
                                then lex' (pos + 2) (Le :: acc)
                                else lex' (pos + 1) (Lt :: acc)
                        | '>' -> 
                                if pos + 1 < String.length input && input.[pos + 1] = '='
                                then lex' (pos + 2) (Ge :: acc)
                                else lex' (pos + 1) (Gt :: acc)
                        | '=' -> 
                                if pos + 1 < String.length input && input.[pos + 1] = '='
                                then lex' (pos + 2) (EqEq :: acc)
                                else lex' (pos + 1) (Eq :: acc)
                        | '!' -> 
                                if pos + 1 < String.length input && input.[pos + 1] = '='
                                then lex' (pos + 2) (Ne :: acc)
                                else raise (CompileError (Lex "Unexpected '!'"))
                        | '+' -> lex' (pos + 1) (Plus :: acc)
                        | '-' -> lex' (pos + 1) (Minus :: acc)
                        | '*' -> lex' (pos + 1) (Star :: acc)
                        | '/' -> lex' (pos + 1) (Slash :: acc)
                        | '%' -> lex' (pos + 1) (Percent :: acc)
                        | '(' -> lex' (pos + 1) (LParen :: acc)
                        | ')' -> lex' (pos + 1) (RParen :: acc)
                        | '{' -> lex' (pos + 1) (LBrace :: acc)
                        | '}' -> lex' (pos + 1) (RBrace :: acc)
                        | ',' -> lex' (pos + 1) (Comma :: acc)
                        | ';' -> lex' (pos + 1) (Semicolon :: acc)
                        | '[' -> lex' (pos + 1) (LBracket :: acc)
                        | ']' -> lex' (pos + 1) (RBracket :: acc)
                        | '.' -> lex' (pos + 1) (Dot :: acc)
                        | '"' -> 
                                let rec collect_string pos' acc =
                                        if pos' >= String.length input then raise (CompileError (Lex " Unterminated string"))
                                        else if input.[pos'] = '"' then (pos' + 1, acc)
                                        else collect_string (pos' + 1) (acc ^ String.make 1 input.[pos'])
                                in
                                let (new_pos, s) = collect_string (pos + 1) "" in 
                                lex' new_pos ( StrLiteral s :: acc)
                        | c when is_digit c -> 
                                let rec collect_number pos' acc =
                                        if pos' >= String.length input || not (is_digit input.[pos'])
                                        then (pos', acc)
                                        else collect_number (pos' + 1) (acc * 10 + (Char.code input.[pos']  - Char.code '0'))
                                in
                                let (new_pos, n ) = collect_number pos 0 in
                                lex' new_pos (Number (Int64.of_int n) :: acc)
                        | c when is_ident_start c ->
                                let rec collect_ident pos' acc =
                                        if pos' >= String.length  input || not (is_ident_continue input.[pos']) 
                                        then (pos', acc)
                                        else collect_ident (pos' + 1) (acc ^ String.make 1 input.[pos'])
                                in 
                                let (new_pos, ident) = collect_ident (pos + 1) (String.make 1 c) in 
                                let token =
                                        match ident with
                                        | "fn" -> Fn
                                        | "var" -> Var
                                        | "let" -> Let
                                        | "if" -> If
                                        | "else" -> Else
                                        | "while" -> While
                                        | "return" -> Return
                                        | "print" -> Print 
                                        | "true" -> BoolLiteral true
                                        | "false" -> BoolLiteral false
                                        | _ -> Ident ident
                                in
                                lex' new_pos (token :: acc)
                        | c -> raise (CompileError (Lex (Printf.sprintf "Unexpected character '%c'" c)))

in 
try Ok (lex' 0 []) with CompileError e -> Error e


(* lexer.ml *)

let string_of_token = function
  | Fn -> "fn"
  | Let -> "let"
  | Var -> "var"
  | If -> "if"
  | Else -> "else"
  | While -> "while"
  | Return -> "return"
  | Print -> "print"
  | Ident s -> Printf.sprintf "identifier '%s'" s
  | Number n -> Printf.sprintf "number %Ld" n
  | StrLiteral s -> Printf.sprintf "string literal '%s'" s
  | BoolLiteral b -> Printf.sprintf "boolean %b" b
  | Plus -> "+"
  | Minus -> "-"
  | Star -> "*"
  | Slash -> "/"
  | Percent -> "%"
  | Lt -> "<"
  | Le -> "<="
  | Gt -> ">"
  | Ge -> ">="
  | EqEq -> "=="
  | Ne -> "!="
  | Eq -> "="
  | LParen -> "("
  | RParen -> ")"
  | LBrace -> "{"
  | RBrace -> "}"
  | LBracket -> "["
  | RBracket -> "]"
  | Comma -> ","
  | Semicolon -> ";"
  | Dot -> "."
  | EOF -> "EOF" *)


open Error

(*
   Lexer for our toy language.
   Produces a token list from an input string.
*)

type token =
  | Fn | Let | Var | If | Else | While | Return | Print
  | Ident of string
  | Number of int64
  | StrLiteral of string
  | BoolLiteral of bool
  | Plus | Minus | Star | Slash | Percent
  | Lt | Le | Gt | Ge | EqEq | Eq | Ne
  | LParen | RParen | LBrace | RBrace | LBracket | RBracket
  | Comma | Semicolon | Dot | EOF

let keywords = [
  ("fn", Fn); ("let", Let); ("var", Var); ("if", If); ("else", Else);
  ("while", While); ("return", Return); ("print", Print)
]

let is_whitespace = function
  | ' ' | '\t' | '\n' | '\r' -> true
  | _ -> false

let is_digit c = c >= '0' && c <= '9'
let is_alpha c =
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c = '_'
let is_alphanum c = is_alpha c || is_digit c

let lex src : token list =
  let len = String.length src in
  let pos = ref 0 in
  let tokens = ref [] in

  let error_at msg =
    (* wrap in our Error module, or fail *)
    failwith (Error.string_of_error (Lex msg))
  in

  let peek () = if !pos < len then src.[!pos] else '\000' in
  let advance () = if !pos < len then incr pos; peek () in

  let rec skip_whitespace () =
    if !pos < len && is_whitespace (peek ()) then (ignore (advance ()); skip_whitespace ())
  in

  let lex_number () =
    let start = !pos in
    while !pos < len && is_digit (peek ()) do ignore (advance ()) done;
    let s = String.sub src start (!pos - start) in
    try Number (Int64.of_string s)
    with _ -> error_at ("invalid number: " ^ s)
  in

  let lex_identifier () =
    let start = !pos in
    while !pos < len && is_alphanum (peek ()) do ignore (advance ()) done;
    let s = String.sub src start (!pos - start) in
    match List.assoc_opt s keywords with
    | Some kw -> kw
    | None    -> Ident s
  in

  let lex_string () =
    ignore (advance ());
    let start = !pos in
    while !pos < len && peek () <> '"' do ignore (advance ()) done;
    if !pos >= len then error_at "unterminated string literal";
    let s = String.sub src start (!pos - start) in
    ignore (advance ()); 
    StrLiteral s
  in

  let rec aux () =
    skip_whitespace ();
    if !pos >= len then () else
    let tok =
      match peek () with
      | c when is_digit c       -> lex_number ()
      | c when is_alpha c       -> lex_identifier ()
      | '"'                    -> lex_string ()
      | '+'                     -> ignore (advance ()); Plus
      | '-'                     -> ignore (advance ()); Minus
      | '*'                     -> ignore (advance ()); Star
      | '/'                     -> ignore (advance ()); Slash
      | '%'                     -> ignore (advance ()); Percent
      | '<'                     -> ignore (advance ()); if peek () = '=' then (ignore (advance ()); Le) else Lt
      | '>'                     -> ignore (advance ()); if peek () = '=' then (ignore (advance ()); Ge) else Gt
      | '='                     -> ignore (advance ()); if peek () = '=' then (ignore (advance ()); EqEq) else Eq
      | '!'                     -> ignore (advance ()); if peek () = '=' then (ignore (advance ()); Ne) else error_at "unexpected !"
      | '('                     -> ignore (advance ()); LParen
      | ')'                     -> ignore (advance ()); RParen
      | '{'                     -> ignore (advance ()); LBrace
      | '}'                     -> ignore (advance ()); RBrace
      | '['                     -> ignore (advance ()); LBracket
      | ']'                     -> ignore (advance ()); RBracket
      | ','                     -> ignore (advance ()); Comma
      | ';'                     -> ignore (advance ()); Semicolon
      | '.'                     -> ignore (advance ()); Dot
      | _                       -> error_at (Printf.sprintf "unexpected char '%c'" (peek ()))
    in
    tokens := tok :: !tokens;
    aux ()
  in
  aux ();
  List.rev (EOF :: !tokens)

(* expose a helper for printing tokens *)
let string_of_token = function
  | Fn -> "Fn" | Let -> "Let" | Var -> "Var" | If -> "If" | Else -> "Else"
  | While -> "While" | Return -> "Return" | Print -> "Print"
  | Ident s -> "Ident(" ^ s ^ ")"
  | Number n -> "Number(" ^ Int64.to_string n ^ ")"
  | StrLiteral s -> "StrLiteral(\"" ^ s ^ "\")"
  | BoolLiteral b -> "BoolLiteral(" ^ string_of_bool b ^ ")"
  | Plus -> "+" | Minus -> "-" | Star -> "*" | Slash -> "/" | Percent -> "%"
  | Lt -> "<" | Le -> "<=" | Gt -> ">" | Ge -> ">="
  | EqEq -> "==" | Eq -> "=" | Ne -> "!="
  | LParen -> "(" | RParen -> ")" | LBrace -> "{" | RBrace -> "}"
  | LBracket -> "[" | RBracket -> "]" | Comma -> "," | Semicolon -> ";" | Dot -> "."
  | EOF -> "EOF"
