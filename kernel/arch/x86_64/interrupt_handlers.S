.code64
.intel_syntax noprefix

.macro error_interrupt_handler number
  .align 16
  .global interrupt_handler\number
  interrupt_handler\number:
    push qword ptr \number                   
    jmp common_interrupt_handler
.endm

.macro no_error_interrupt_handler number
  .align 16
  .global interrupt_handler\number
  interrupt_handler\number:
    push qword ptr 0
    push qword ptr \number                   
    jmp common_interrupt_handler
.endm

.extern interrupt_dispatch

common_interrupt_handler:
  push rax
  push rbx
  push rcx
  push rdx
  push rsi
  push rdi
  push r8
  push r9
  push r10
  push r11
  push r12
  push r13
  push r14
  push r15

  mov rdi, rsp
  call interrupt_dispatch

  pop r15
  pop r14
  pop r13
  pop r12
  pop r11
  pop r10
  pop r9
  pop r8
  pop rdi
  pop rsi
  pop rdx
  pop rcx
  pop rbx
  pop rax

  add rsp, 16
  iretq

no_error_interrupt_handler 0
no_error_interrupt_handler 1
no_error_interrupt_handler 2
no_error_interrupt_handler 3
no_error_interrupt_handler 4
no_error_interrupt_handler 5
no_error_interrupt_handler 6
no_error_interrupt_handler 7
error_interrupt_handler 8
no_error_interrupt_handler 9
error_interrupt_handler 10
error_interrupt_handler 11
error_interrupt_handler 12
error_interrupt_handler 13
error_interrupt_handler 14
no_error_interrupt_handler 15
no_error_interrupt_handler 16
error_interrupt_handler 17
no_error_interrupt_handler 18
no_error_interrupt_handler 19
no_error_interrupt_handler 20
no_error_interrupt_handler 21
no_error_interrupt_handler 22
no_error_interrupt_handler 23
no_error_interrupt_handler 24
no_error_interrupt_handler 25
no_error_interrupt_handler 26
no_error_interrupt_handler 27
no_error_interrupt_handler 28
no_error_interrupt_handler 29
error_interrupt_handler 30
no_error_interrupt_handler 31

# `.altmacro` ensures that %i gets resolved to it's numeric value
.altmacro 
.set i, 32
.rept 256 - 32
    no_error_interrupt_handler %i
    .set i, i+1
.endr
