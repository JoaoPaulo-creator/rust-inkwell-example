; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.2 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca [0 x i32], align 4
  %printi = call i32 (ptr, ...) @printf(ptr @fmt, i32 0)
  %index_ptr = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 2
  %index_load = load i32, ptr %index_ptr, align 4
  %printi1 = call i32 (ptr, ...) @printf(ptr @fmt.1, i32 %index_load)
  %index_ptr2 = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 0
  store i32 10, ptr %index_ptr2, align 4
  %index_ptr3 = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 0
  %index_load4 = load i32, ptr %index_ptr3, align 4
  %printi5 = call i32 (ptr, ...) @printf(ptr @fmt.2, i32 %index_load4)
  ret i32 0
}
