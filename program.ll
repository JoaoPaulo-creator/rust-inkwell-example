; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %arr = alloca [3 x i32], align 4
  %elem_ptr = getelementptr inbounds [3 x i32], ptr %arr, i32 0, i32 0
  store i32 1, ptr %elem_ptr, align 4
  %elem_ptr1 = getelementptr inbounds [3 x i32], ptr %arr, i32 0, i32 1
  store i32 2, ptr %elem_ptr1, align 4
  %elem_ptr2 = getelementptr inbounds [3 x i32], ptr %arr, i32 0, i32 2
  store i32 3, ptr %elem_ptr2, align 4
  %arr3 = alloca ptr, align 8
  store ptr %arr, ptr %arr3, align 8
  %arr_size = alloca i32, align 4
  store i32 3, ptr %arr_size, align 4
  %load_array_ptr = load ptr, ptr %arr3, align 8
  %index_ptr = getelementptr inbounds [0 x i32], ptr %load_array_ptr, i32 0, i32 0
  %index_load = load i32, ptr %index_ptr, align 4
  %print_call = call i32 (ptr, ...) @printf(ptr @fmt, i32 %index_load)
  %load_size = load i32, ptr %arr_size, align 4
  %print_call4 = call i32 (ptr, ...) @printf(ptr @fmt.1, i32 %load_size)
  ret i32 0
}
