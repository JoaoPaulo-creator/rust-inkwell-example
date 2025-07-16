open Ast
open Error
open Lexer

type parser = {
  tokens: token list;
  mutable pos: int;
}

let make_parser tokens = { tokens; pos = 0 }

let peek p = List.nth p.tokens p.pos
let eat p = if p.pos < List.length p.tokens then p.pos <- p.pos + 1
let expect p expected =
  if peek p = expected then (eat p; Ok ())
  else Error (Parse (Printf.sprintf "Expected %s, found %s" (string_of_token expected) (string_of_token (peek p))))


(* helper *)
let rec parse_many_until p stop_token parse_one acc =
  if peek p = stop_token then (
    eat p;
    Ok (List.rev acc)
  ) else 
    match parse_one p with
    | Error e -> Error e
    | Ok item ->
      if peek p = Semicolon then eat p;
      parse_many_until p stop_token parse_one (item :: acc)


let rec parse_program p =
  let rec collect_fns acc =
    if peek p = Fn then 
      match parse_function p with 
      | Error e -> Error e 
      | Ok fn -> collect_fns (fn :: acc)
    else 
      Ok (List.rev acc)
  in
  match collect_fns [] with 
  | Error e -> Error e 
  | Ok functions -> 
    match parse_many_until p EOF parse_statement [] with
    | Error e -> Error e
    | Ok statements -> Ok {functions; statements}


and parse_function p =
  match expect p Fn with
  | Ok () ->
      let name = match peek p with Ident n -> n | _ -> raise (CompileError (Parse "Expected function name")) in
      eat p;
      (match expect p LParen with
       | Ok () ->
           let params = parse_params p [] in
           (match expect p RParen with
            | Ok () ->
                (match parse_block p with
                 | Ok body -> Ok { name; params; body }
                 | Error e -> Error e)
            | Error e -> Error e)
       | Error e -> Error e)
  | Error e -> Error e

and parse_params p acc =
  if peek p = RParen then List.rev acc
  else
    let name = match peek p with Ident n -> n | _ -> raise (CompileError (Parse "Expected parameter name")) in
    eat p;
    let acc' = name :: acc in
    if peek p = Comma then (eat p; parse_params p acc')
    else acc'

and parse_block p =
  match expect p LBrace with
  | Ok () ->
       (match parse_many_until p RBrace parse_statement [] with
       | Error e -> Error e
       | Ok stmts -> Ok stmts
       )
  | Error e -> Error e

and parse_statement p =
  match peek p with
  | Var ->
      eat p;
      let name = match peek p with Ident n -> n | _ -> raise (CompileError (Parse "Expected var name")) in
      eat p;
      (match expect p Eq with
       | Ok () ->
           (match parse_expr p with
            | Ok expr -> Ok (VarDecl (name, expr))
            | Error e -> Error e)
       | Error e -> Error e)
  | Let ->
      eat p;
      let name = match peek p with Ident n -> n | _ -> raise (CompileError (Parse "Expected let name")) in
      eat p;
      (match expect p Eq with
       | Ok () ->
           (match parse_expr p with
            | Ok expr -> Ok (LetDecl (name, expr))
            | Error e -> Error e)
       | Error e -> Error e)
  | If ->
      eat p;
      (match expect p LParen with
       | Ok () ->
           (match parse_expr p with
            | Ok cond ->
                (match expect p RParen with
                 | Ok () ->
                     (match parse_block p with
                      | Ok then_branch ->
                          let else_branch =
                            if peek p = Else then (
                              eat p;
                              match parse_block p with
                              | Ok block -> Some block
                              | Error _ -> None
                            ) else None
                          in
                          Ok (If (cond, then_branch, else_branch))
                      | Error e -> Error e)
                 | Error e -> Error e)
            | Error e -> Error e)
       | Error e -> Error e)
  | While ->
      eat p;
      (match expect p LParen with
       | Ok () ->
           (match parse_expr p with
            | Ok cond ->
                (match expect p RParen with
                 | Ok () ->
                     (match parse_block p with
                      | Ok body -> Ok (While (cond, body))
                      | Error e -> Error e)
                 | Error e -> Error e)
            | Error e -> Error e)
       | Error e -> Error e)
  | Return ->
      eat p;
      (match parse_expr p with
       | Ok expr -> Ok (Return expr)
       | Error e -> Error e)
  | Print ->
      eat p;
      (match parse_expr p with
       | Ok expr -> Ok (Print expr)
       | Error e -> Error e)
  | Ident _ ->
      (match parse_expr p with
       | Ok expr ->
           if peek p = Eq then (
             eat p;
             match parse_expr p with
             | Ok value ->
                 (match expr with
                  | Variable name -> Ok (Assign (name, value))
                  | Index (array, index) -> Ok (IndexAssign (array, index, value))
                  | _ -> Error (Parse "Left-hand side of assignment must be a variable or array index"))
             | Error e -> Error e
           ) else Ok (ExprStmt expr)
       | Error e -> Error e)
  | _ ->
      (match parse_expr p with
       | Ok expr -> Ok (ExprStmt expr)
       | Error e -> Error e)

and parse_expr p = parse_equality p

and parse_equality p =
  match parse_comparison p with
  | Ok lhs ->
      let rec loop lhs =
        match peek p with
        | EqEq ->
            eat p;
            (match parse_comparison p with
             | Ok rhs -> loop (Binary (Eq, lhs, rhs))
             | Error e -> Error e)
        | Ne ->
            eat p;
            (match parse_comparison p with
             | Ok rhs -> loop (Binary (Ne, lhs, rhs))
             | Error e -> Error e)
        | _ -> Ok lhs
      in
      loop lhs
  | Error e -> Error e

and parse_comparison p =
  match parse_addition p with
  | Ok lhs ->
      let rec loop lhs =
        match peek p with
        | Lt ->
            eat p;
            (match parse_addition p with
             | Ok rhs -> loop (Binary (Lt, lhs, rhs))
             | Error e -> Error e)
        | Le ->
            eat p;
            (match parse_addition p with
             | Ok rhs -> loop (Binary (Le, lhs, rhs))
             | Error e -> Error e)
        | Gt ->
            eat p;
            (match parse_addition p with
             | Ok rhs -> loop (Binary (Gt, lhs, rhs))
             | Error e -> Error e)
        | Ge ->
            eat p;
            (match parse_addition p with
             | Ok rhs -> loop (Binary (Ge, lhs, rhs))
             | Error e -> Error e)
        | _ -> Ok lhs
      in
      loop lhs
  | Error e -> Error e

and parse_addition p =
  match parse_term p with
  | Ok lhs ->
      let rec loop lhs =
        match peek p with
        | Plus ->
            eat p;
            (match parse_term p with
             | Ok rhs -> loop (Binary (Add, lhs, rhs))
             | Error e -> Error e)
        | Minus ->
            eat p;
            (match parse_term p with
             | Ok rhs -> loop (Binary (Sub, lhs, rhs))
             | Error e -> Error e)
        | _ -> Ok lhs
      in
      loop lhs
  | Error e -> Error e

and parse_term p =
  match parse_factor p with
  | Ok lhs ->
      let rec loop lhs =
        match peek p with
        | Star ->
            eat p;
            (match parse_factor p with
             | Ok rhs -> loop (Binary (Mul, lhs, rhs))
             | Error e -> Error e)
        | Slash ->
            eat p;
            (match parse_factor p with
             | Ok rhs -> loop (Binary (Div, lhs, rhs))
             | Error e -> Error e)
        | Percent ->
            eat p;
            (match parse_factor p with
             | Ok rhs -> loop (Binary (Rem, lhs, rhs))
             | Error e -> Error e)
        | _ -> Ok lhs
      in
      loop lhs
  | Error e -> Error e

and parse_factor p =
  match peek p with
  | LBracket ->
      eat p;
      let rec parse_elems acc =
        if peek p = RBracket then Ok (List.rev acc)
        else
          match parse_expr p with
          | Ok e ->
              let acc' = e :: acc in
              if peek p = Comma then (eat p; parse_elems acc')
              else parse_elems acc'
          | Error e -> Error e
      in
      (match parse_elems [] with
       | Ok elems ->
           (match expect p RBracket with
            | Ok () -> Ok (ArrayLiteral elems)
            | Error e -> Error e)
       | Error e -> Error e)
  | Number n ->
      eat p;
      Ok (Number n)
  | BoolLiteral b ->
      eat p;
      Ok (Bool b)
  | StrLiteral s ->
      eat p;
      Ok (StrLiteral s)
  | Ident name ->
      eat p;
      if peek p = LParen then (
        eat p;
        let rec parse_args acc =
          if peek p = RParen then Ok (List.rev acc)
          else
            match parse_expr p with
            | Ok arg ->
                let acc' = arg :: acc in
                if peek p = Comma then (eat p; parse_args acc')
                else parse_args acc'
            | Error e -> Error e
        in
        (match parse_args [] with
         | Ok args ->
             (match expect p RParen with
              | Ok () -> Ok (Call (name, args))
              | Error e -> Error e)
         | Error e -> Error e)
      ) else Ok (Variable name)
  | LParen ->
      eat p;
      (match parse_expr p with
       | Ok e ->
           (match expect p RParen with
            | Ok () -> Ok e
            | Error e -> Error e)
       | Error e -> Error e)
  | other ->
      Error (Parse (Printf.sprintf "Unexpected token in factor: %s" (string_of_token other)))