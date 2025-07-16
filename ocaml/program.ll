; ModuleID = 'toy'
source_filename = "toy"

@fmt_int = global [6 x i8] c"%d\\n\\0"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 2, i32* %a, align 4
  %b = alloca i32, align 4
  store i32 2, i32* %b, align 4
  %variable = load i32, i32* %a, align 4
  %variable1 = load i32, i32* %b, align 4
  %addtmp = add i32 %variable, %variable1
  %c = alloca i32, align 4
  store i32 %addtmp, i32* %c, align 4
  %variable2 = load i32, i32* %c, align 4
  %print_call = call i32 (ptr, ...) @printf(i8* getelementptr inbounds ([7 x i8], [6 x i8]* @fmt_int, i32 0, i32 0), i32 %variable2)
  ret i32 0
}
