; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.2 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca [4 x i32], align 4
  %elem_ptr = getelementptr inbounds [4 x i32], ptr %a, i32 0, i32 0
  store i32 1, ptr %elem_ptr, align 4
  %elem_ptr1 = getelementptr inbounds [4 x i32], ptr %a, i32 0, i32 1
  store i32 2, ptr %elem_ptr1, align 4
  %elem_ptr2 = getelementptr inbounds [4 x i32], ptr %a, i32 0, i32 2
  store i32 3, ptr %elem_ptr2, align 4
  %elem_ptr3 = getelementptr inbounds [4 x i32], ptr %a, i32 0, i32 3
  store i32 4, ptr %elem_ptr3, align 4
  %printi = call i32 (ptr, ...) @printf(ptr @fmt, i32 0)
  %index_ptr = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 2
  %index_load = load i32, ptr %index_ptr, align 4
  %printi4 = call i32 (ptr, ...) @printf(ptr @fmt.1, i32 %index_load)
  %index_ptr5 = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 0
  store i32 10, ptr %index_ptr5, align 4
  %index_ptr6 = getelementptr inbounds [0 x i32], ptr %a, i32 0, i32 0
  %index_load7 = load i32, ptr %index_ptr6, align 4
  %printi8 = call i32 (ptr, ...) @printf(ptr @fmt.2, i32 %index_load7)
  ret i32 0
}
