; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @twoSum(ptr %0, i32 %1, i32 %2) {
entry:
  %arr = alloca ptr, align 8
  store ptr %0, ptr %arr, align 8
  %arr_size = alloca i32, align 4
  store i32 %1, ptr %arr_size, align 4
  %target = alloca i32, align 4
  store i32 %2, ptr %target, align 4
  %left = alloca i32, align 4
  store i32 0, ptr %left, align 4
  %load_size = load i32, ptr %arr_size, align 4
  %right = alloca i32, align 4
  store i32 %load_size, ptr %right, align 4
  br label %loop

loop:                                             ; preds = %body, %entry
  %left1 = load i32, ptr %left, align 4
  %right2 = load i32, ptr %right, align 4
  %lttmp = icmp slt i32 %left1, %right2
  %bool2int = zext i1 %lttmp to i32
  %whilecond = icmp ne i32 %bool2int, 0
  br i1 %whilecond, label %body, label %after

after:                                            ; preds = %loop
  ret i32 0

body:                                             ; preds = %loop
  %left3 = load i32, ptr %left, align 4
  %addtmp = add i32 %left3, 1
  store i32 %addtmp, ptr %left, align 4
  br label %loop
}

define i32 @main() {
entry:
  %arr = alloca [5 x i32], align 4
  %elem_ptr = getelementptr inbounds [5 x i32], ptr %arr, i32 0, i32 0
  store i32 1, ptr %elem_ptr, align 4
  %elem_ptr1 = getelementptr inbounds [5 x i32], ptr %arr, i32 0, i32 1
  store i32 2, ptr %elem_ptr1, align 4
  %elem_ptr2 = getelementptr inbounds [5 x i32], ptr %arr, i32 0, i32 2
  store i32 3, ptr %elem_ptr2, align 4
  %elem_ptr3 = getelementptr inbounds [5 x i32], ptr %arr, i32 0, i32 3
  store i32 4, ptr %elem_ptr3, align 4
  %elem_ptr4 = getelementptr inbounds [5 x i32], ptr %arr, i32 0, i32 4
  store i32 5, ptr %elem_ptr4, align 4
  %arr5 = alloca ptr, align 8
  store ptr %arr, ptr %arr5, align 8
  %load_array_ptr = load ptr, ptr %arr5, align 8
  %calltmp = call i32 @twoSum(ptr %load_array_ptr, i32 5, i32 6)
  %print_call = call i32 (ptr, ...) @printf(ptr @fmt, i32 %calltmp)
  ret i32 0
}
