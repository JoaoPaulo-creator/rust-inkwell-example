; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @makeArray() {
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
  %a4 = alloca ptr, align 8
  store ptr %a, ptr %a4, align 8
  %print_call = call i32 (ptr, ...) @printf(ptr @fmt, i32 4)
  ret i32 0
}

define i32 @main() {
entry:
  %calltmp = call i32 @makeArray()
  ret i32 0
}
