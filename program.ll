; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.1 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@str = private unnamed_addr constant [5 x i8] c"done\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %i = alloca i32, align 4
  store i32 3, ptr %i, align 4
  br label %loop

loop:                                             ; preds = %body, %entry
  %i1 = load i32, ptr %i, align 4
  %lttmp = icmp slt i32 %i1, 10
  %bool2int = zext i1 %lttmp to i32
  %whilecond = icmp ne i32 %bool2int, 0
  br i1 %whilecond, label %body, label %after

after:                                            ; preds = %loop
  %printstr = call i32 (ptr, ...) @printf(ptr @fmt.1, ptr @str)
  ret i32 0

body:                                             ; preds = %loop
  %i2 = load i32, ptr %i, align 4
  %printi = call i32 (ptr, ...) @printf(ptr @fmt, i32 %i2)
  %i3 = load i32, ptr %i, align 4
  %addtmp = add i32 %i3, 1
  store i32 %addtmp, ptr %i, align 4
  br label %loop
}
