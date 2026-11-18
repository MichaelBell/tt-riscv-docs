/* TinyQV (ttsky25a) memory map, mirrors tinyQV-sdk/memmap.
   Code runs XIP from the QSPI flash at 0; data/stack live in PSRAM A.
   FLASH is limited to 1 MiB so the image cannot clobber other data
   stored higher up in the Pmod flash. */
MEMORY
{
  FLASH (rx) : ORIGIN = 0x0000000, LENGTH = 1M
  /* 0x1000000-0x10003FF is reserved: the core saves interrupt context at gp-0x200 */
  RAM  (rw)  : ORIGIN = 0x1000400, LENGTH = 8M - 0x400
}

ENTRY(_start)

SECTIONS
{
  .text : {
    KEEP(*(.boot))          /* must be at address 0: reset/trap/interrupt entry */
    *(.text .text.*)
  } > FLASH

  .rodata : ALIGN(4) {
    *(.srodata .srodata.*)
    *(.rodata .rodata.*)
  } > FLASH

  .data : ALIGN(4) {
    __sdata = .;
    *(.sdata .sdata.*)
    *(.data .data.*)
    . = ALIGN(4);
    __edata = .;
  } > RAM AT > FLASH
  __sidata = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(4) {
    __sbss = .;
    *(.sbss .sbss.*)
    *(.bss .bss.*)
    . = ALIGN(4);
    __ebss = .;
  } > RAM

  __heap_start = ALIGN(__ebss, 16);
  __stack_top = ORIGIN(RAM) + LENGTH(RAM);   /* 0x1800000, end of PSRAM A */

  /* TinyQV hardwires gp to 0x1000400; lld may relax accesses near it */
  __global_pointer$ = 0x1000400;

  /DISCARD/ : { *(.eh_frame .eh_frame_hdr .riscv.attributes) }
}
