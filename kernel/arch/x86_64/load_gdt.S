# NOTE:
# the placement in the `gdt` affects these values
.set KERNEL_CODE_SEGMENT, 0x08
.set KERNEL_DATA_SEGMENT, 0x10

.intel_syntax noprefix
.code64
.global load_gdt

load_gdt:
  lgdt [rdi]
  mov ax, KERNEL_DATA_SEGMENT
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax
  pop rdi
  push KERNEL_CODE_SEGMENT
  push rdi
  retfq
