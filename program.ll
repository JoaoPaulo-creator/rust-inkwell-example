; ModuleID = 'toy'
source_filename = "toy"

@fmt = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

declare i32 @printf(ptr, ...)

define i32 @add(i32 %0, i32 %1) {
entry:
  %x = alloca i32, align 4
  store i32 %0, ptr %x, align 4
  %y = alloca i32, align 4
  store i32 %1, ptr %y, align 4
  %x1 = load i32, ptr %x, align 4
  %y2 = load i32, ptr %y, align 4
  %addtmp = add i32 %x1, %y2
  ret i32 %addtmp
  ret i32 0
}

define i32 @main() {
entry:
  %calltmp = call i32 @add(i32 3, i32 4)
  %z = alloca i32, align 4
  store i32 %calltmp, ptr %z, align 4
  %z1 = load i32, ptr %z, align 4
  %eqtmp = icmp eq i32 %z1, 7
  %bool2int = zext i1 %eqtmp to i32
  %ifcond = icmp ne i32 %bool2int, 0
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %entry
  %z2 = load i32, ptr %z, align 4
  %printi = call i32 (ptr, ...) @printf(ptr @fmt, i32 %z2)
  br label %ifcont

else:                                             ; preds = %entry
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  ret i32 0
}
