; ModuleID = 'toy'
source_filename = "toy"

@bounds_err_fmt = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@bounds_err_fmt.1 = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@bounds_err_fmt.2 = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@bounds_err_fmt.3 = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@bounds_err_fmt.4 = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@bounds_err_fmt.5 = private unnamed_addr constant [42 x i8] c"Index out of bounds: %d (array size: %d)\0A\00", align 1
@fmt.6 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define ptr @twoSum(ptr %0, i32 %1, i32 %2) {
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
  %subtmp = sub i32 %load_size, 1
  %right = alloca i32, align 4
  store i32 %subtmp, ptr %right, align 4
  br label %loop

loop:                                             ; preds = %ifcont, %entry
  %left1 = load i32, ptr %left, align 4
  %right2 = load i32, ptr %right, align 4
  %lttmp = icmp slt i32 %left1, %right2
  %bool2int = zext i1 %lttmp to i32
  %whilecond = icmp ne i32 %bool2int, 0
  br i1 %whilecond, label %body, label %after

after:                                            ; preds = %loop
  %ret_array42 = alloca [2 x i32], align 4
  %elem_ptr43 = getelementptr inbounds [2 x i32], ptr %ret_array42, i32 0, i32 0
  store i32 0, ptr %elem_ptr43, align 4
  %elem_ptr44 = getelementptr inbounds [2 x i32], ptr %ret_array42, i32 0, i32 1
  store i32 0, ptr %elem_ptr44, align 4
  ret ptr %ret_array42

body:                                             ; preds = %loop
  %load_array_ptr = load ptr, ptr %arr, align 8
  %left3 = load i32, ptr %left, align 4
  %idx_ge_zero = icmp sge i32 %left3, 0
  %idx_lt_size = icmp slt i32 %left3, 0
  %idx_valid = and i1 %idx_ge_zero, %idx_lt_size
  br i1 %idx_valid, label %continue, label %bounds_error

bounds_error:                                     ; preds = %body
  %print_bounds_error = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt, i32 %left3, i32 0)
  ret i32 1

continue:                                         ; preds = %body
  %index_ptr = getelementptr inbounds [0 x i32], ptr %load_array_ptr, i32 0, i32 %left3
  %index_load = load i32, ptr %index_ptr, align 4
  %load_array_ptr4 = load ptr, ptr %arr, align 8
  %right5 = load i32, ptr %right, align 4
  %idx_ge_zero6 = icmp sge i32 %right5, 0
  %idx_lt_size7 = icmp slt i32 %right5, 0
  %idx_valid8 = and i1 %idx_ge_zero6, %idx_lt_size7
  br i1 %idx_valid8, label %continue10, label %bounds_error9

bounds_error9:                                    ; preds = %continue
  %print_bounds_error11 = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt.1, i32 %right5, i32 0)
  ret i32 1

continue10:                                       ; preds = %continue
  %index_ptr12 = getelementptr inbounds [0 x i32], ptr %load_array_ptr4, i32 0, i32 %right5
  %index_load13 = load i32, ptr %index_ptr12, align 4
  %addtmp = add i32 %index_load, %index_load13
  %sum = alloca i32, align 4
  store i32 %addtmp, ptr %sum, align 4
  %sum14 = load i32, ptr %sum, align 4
  %target15 = load i32, ptr %target, align 4
  %eqtmp = icmp eq i32 %sum14, %target15
  %bool2int16 = zext i1 %eqtmp to i32
  %ifcond = icmp ne i32 %bool2int16, 0
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %continue10
  %ret_array = alloca [2 x i32], align 4
  %load_array_ptr17 = load ptr, ptr %arr, align 8
  %left18 = load i32, ptr %left, align 4
  %idx_ge_zero19 = icmp sge i32 %left18, 0
  %idx_lt_size20 = icmp slt i32 %left18, 0
  %idx_valid21 = and i1 %idx_ge_zero19, %idx_lt_size20
  br i1 %idx_valid21, label %continue23, label %bounds_error22

else:                                             ; preds = %continue10
  %left38 = load i32, ptr %left, align 4
  %addtmp39 = add i32 %left38, 1
  store i32 %addtmp39, ptr %left, align 4
  %right40 = load i32, ptr %right, align 4
  %subtmp41 = sub i32 %right40, 1
  store i32 %subtmp41, ptr %right, align 4
  br label %ifcont

ifcont:                                           ; preds = %else
  br label %loop

bounds_error22:                                   ; preds = %then
  %print_bounds_error24 = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt.2, i32 %left18, i32 0)
  ret i32 1

continue23:                                       ; preds = %then
  %index_ptr25 = getelementptr inbounds [0 x i32], ptr %load_array_ptr17, i32 0, i32 %left18
  %index_load26 = load i32, ptr %index_ptr25, align 4
  %elem_ptr = getelementptr inbounds [2 x i32], ptr %ret_array, i32 0, i32 0
  store i32 %index_load26, ptr %elem_ptr, align 4
  %load_array_ptr27 = load ptr, ptr %arr, align 8
  %right28 = load i32, ptr %right, align 4
  %idx_ge_zero29 = icmp sge i32 %right28, 0
  %idx_lt_size30 = icmp slt i32 %right28, 0
  %idx_valid31 = and i1 %idx_ge_zero29, %idx_lt_size30
  br i1 %idx_valid31, label %continue33, label %bounds_error32

bounds_error32:                                   ; preds = %continue23
  %print_bounds_error34 = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt.3, i32 %right28, i32 0)
  ret i32 1

continue33:                                       ; preds = %continue23
  %index_ptr35 = getelementptr inbounds [0 x i32], ptr %load_array_ptr27, i32 0, i32 %right28
  %index_load36 = load i32, ptr %index_ptr35, align 4
  %elem_ptr37 = getelementptr inbounds [2 x i32], ptr %ret_array, i32 0, i32 1
  store i32 %index_load36, ptr %elem_ptr37, align 4
  ret ptr %ret_array
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
  %arr_size = alloca i32, align 4
  store i32 5, ptr %arr_size, align 4
  %load_array_ptr = load ptr, ptr %arr5, align 8
  %calltmp = call ptr @twoSum(ptr %load_array_ptr, i32 5, i32 6)
  %result = alloca [2 x i32], align 4
  store ptr %calltmp, ptr %result, align 8
  %result_size = alloca i32, align 4
  store i32 2, ptr %result_size, align 4
  %load_array_ptr6 = load ptr, ptr %result, align 8
  br i1 true, label %continue, label %bounds_error

bounds_error:                                     ; preds = %entry
  %print_bounds_error = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt.4, i32 0, i32 2)
  ret i32 1

continue:                                         ; preds = %entry
  %index_ptr = getelementptr inbounds [2 x i32], ptr %load_array_ptr6, i32 0, i32 0
  %index_load = load i32, ptr %index_ptr, align 4
  %print_call = call i32 (ptr, ...) @printf(ptr @fmt, i32 %index_load)
  %load_array_ptr7 = load ptr, ptr %result, align 8
  br i1 true, label %continue9, label %bounds_error8

bounds_error8:                                    ; preds = %continue
  %print_bounds_error10 = call i32 (ptr, ...) @printf(ptr @bounds_err_fmt.5, i32 1, i32 2)
  ret i32 1

continue9:                                        ; preds = %continue
  %index_ptr11 = getelementptr inbounds [2 x i32], ptr %load_array_ptr7, i32 0, i32 1
  %index_load12 = load i32, ptr %index_ptr11, align 4
  %print_call13 = call i32 (ptr, ...) @printf(ptr @fmt.6, i32 %index_load12)
  ret i32 0
}
