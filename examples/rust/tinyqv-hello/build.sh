#!/bin/sh
# Build the demo, produce tinyqv-hello.bin (flash image for offset 0) and check the instruction set.
set -e
cd "$(dirname "$0")"
cargo build --release
ELF=target/riscv32ec-tinyqv-none-elf/release/tinyqv-hello
BIN=$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/host: //p')/bin
"$BIN/llvm-objcopy" -O binary "$ELF" tinyqv-hello.bin
SIZE=$(wc -c < tinyqv-hello.bin)
echo "tinyqv-hello.bin: $SIZE bytes"
[ "$SIZE" -lt 1048576 ] || { echo "image >= 1 MiB: would overwrite other flash contents" >&2; exit 1; }
# Instruction check: every mnemonic must be in RV32E base + C + Zcb + Zicond + Zicsr (+ TinyQV mul16).
"$BIN/llvm-objdump" -d --mattr=+e,+c,+zcb,+zicond,+zicsr "$ELF" |
  awk -F'\t' '$1 ~ /^ +[0-9a-f]+: / {print $2}' | sort | uniq -c | sort -rn > isa-histogram.txt
BAD=$(awk '{print $2}' isa-histogram.txt | grep -vxE \
  'c\.[a-z.]+|lui|auipc|jal|jalr|beq|bne|blt|bge|bltu|bgeu|lb|lh|lw|lbu|lhu|sb|sh|sw|addi|slti|sltiu|xori|ori|andi|slli|srli|srai|add|sub|sll|slt|sltu|xor|srl|sra|or|and|fence|ecall|ebreak|mret|wfi|csrr[wsc]i?|czero\.(eqz|nez)|<unknown>|li|mv|j|jr|ret|call|tail|nop|beqz|bnez|blez|bgez|bltz|bgtz|bgt|ble|bgtu|bleu|seqz|snez|sltz|sgtz|not|neg|zext\.b|rdcycle|rdtime|rdinstret|csr[rwsc]|csr[wsc]i' || true)
# Zcb's zext.b/not are fine; zext.h/sext.* would need Zbb and are rejected.
# <unknown> is the .insn-encoded mul16 (opcode 0x33, funct7 2) which LLVM cannot name
if [ -n "$BAD" ]; then echo "unsupported instructions: $BAD" >&2; exit 1; fi
echo "ISA check ok ($(awk '{s+=$1} END {print s}' isa-histogram.txt) instructions)"
