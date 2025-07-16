; ModuleID = 'toy'
source_filename = "toy"

@str = global [10 x i8] c"ola, mundo"
@fmt_str = global [4 x i8] c"%s \0A"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %print_call = call i32 (ptr, ...) @printf(i8* getelementptr inbounds ([5 x i8], [4 x i8]* @fmt_str, i32 0, i32 0), i8* getelementptr inbounds ([11 x i8], [10 x i8]* @str, i32 0, i32 0))
  ret i32 0
}
