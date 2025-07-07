type error =
        | Io of string
        | Lex of string
        | Parse of string
        | Codegen of string


exception CompileError of error

let string_of_error = function 
        | Io msg -> "Io error: " ^ msg
        | Lex msg -> "Lexical error: " ^ msg
        | Parse msg -> "Parse error: " ^ msg
        | Codegen msg -> "Codegen error: " ^ msg

