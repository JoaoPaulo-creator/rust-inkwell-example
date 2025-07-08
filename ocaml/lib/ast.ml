type un_op = Pos | Neg

type bin_op =
        | Add
        | Sub
        | Mul
        | Div
        | Rem
        | Lt
        | Le
        | Gt
        | Ge
        | Eq
        | Ne

type expr =
        | Number of int64
        | Bool of bool
        | StrLiteral of string
        | Variable of string
        | Unary of un_op * expr
        | Binary of bin_op * expr * expr
        | Call of string * expr list
        | ArrayLiteral of expr list
        | Index of expr * expr
        | Length of expr


type statement =
        | VarDecl of string * expr
        | LetDecl of string * expr
        | Assign of string * expr
        | IndexAssign of expr * expr * expr
        | Return of expr
        | Print of expr
        | If of expr * statement list * statement list option
        | While of expr * statement list
        | ExprStmt of expr

type func = {
        name: string;
        params: string list;
        body: statement list;
}

type program = {
        functions: func list;
        statements: statement list;
}

