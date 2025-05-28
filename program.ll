; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@str = private unnamed_addr constant [12 x i8] c"result of a\00", align 1
@fmt.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.2 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@str.3 = private unnamed_addr constant [12 x i8] c"result of b\00", align 1
@fmt.4 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 3, ptr %a, align 4
  %a1 = load i32, ptr %a, align 4
  %multmp = mul i32 %a1, 3
  %b = alloca i32, align 4
  store i32 %multmp, ptr %b, align 4
  %printstr = call i32 (ptr, ...) @printf(ptr @fmt, ptr @str)
  %a2 = load i32, ptr %a, align 4
  %printi = call i32 (ptr, ...) @printf(ptr @fmt.1, i32 %a2)
  %printstr3 = call i32 (ptr, ...) @printf(ptr @fmt.2, ptr @str.3)
  %b4 = load i32, ptr %b, align 4
  %printi5 = call i32 (ptr, ...) @printf(ptr @fmt.4, i32 %b4)
  ret i32 0
}
