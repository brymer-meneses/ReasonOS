[bits 64]
%macro set_error_interrupt_handler 1
global interrupt_handler%1
align 16
interrupt_handler%1:
  push qword %1                   ; push the interrupt number
  jmp common_interrupt_handler
%endmacro


%macro set_no_error_interrupt_handler 1
global interrupt_handler%1
align 16
interrupt_handler%1:
  push qword 0                    ; push 0 error code
  push qword %1                   ; push the interrupt number
  jmp common_interrupt_handler
%endmacro

; this is defined on `interrupt.rs`
extern interrupt_dispatch

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


set_no_error_interrupt_handler 0
set_no_error_interrupt_handler 1
set_no_error_interrupt_handler 2
set_no_error_interrupt_handler 3
set_no_error_interrupt_handler 4
set_no_error_interrupt_handler 5
set_no_error_interrupt_handler 6
set_no_error_interrupt_handler 7
set_error_interrupt_handler 8
set_no_error_interrupt_handler 9
set_error_interrupt_handler 10
set_error_interrupt_handler 11
set_error_interrupt_handler 12
set_error_interrupt_handler 13
set_error_interrupt_handler 14
set_no_error_interrupt_handler 15
set_no_error_interrupt_handler 16
set_error_interrupt_handler 17
set_no_error_interrupt_handler 18
set_no_error_interrupt_handler 19
set_no_error_interrupt_handler 20
set_no_error_interrupt_handler 21
set_no_error_interrupt_handler 22
set_no_error_interrupt_handler 23
set_no_error_interrupt_handler 24
set_no_error_interrupt_handler 25
set_no_error_interrupt_handler 26
set_no_error_interrupt_handler 27
set_no_error_interrupt_handler 28
set_no_error_interrupt_handler 29
set_error_interrupt_handler 30
set_no_error_interrupt_handler 31

%define MAX_IDT_ENTRIES 256

; Make room for the ISRs
%assign i 32
%rep MAX_IDT_ENTRIES - 32
set_no_error_interrupt_handler i
%assign i i+1
%endrep

global INTERRUPT_HANDLERS
INTERRUPT_HANDLERS:
%assign i 0
%rep 256
  dq interrupt_handler%+i
%assign i i+1
%endrep
  
