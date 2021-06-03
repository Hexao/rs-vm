# rs-vm

## Registers

Every register is followed in memory. Exemple: `acc` is after `ip` ect.

Order of register is as following: `ip`, `acc`, `ax`, `bx`, `cx`, `dx`, `ex`, `fx`, `gx`, `hx`, `sp`, `fp`.

### Special registers

- `ip`: instruction pointer (16bits) don't touch please
- `acc`: accumulator, stored results of mathematical operations (add, sub, mul ...)
- `sp`: stack pointer (handle subroutines)
- `fp`: frame pointer (pointer inside stack frame)

### Normal registers

- `a`:
    - `ax`: main register (16bits) can be separated in two 8bits register
    - `ah`: 8bits register, upper byte of `ax`
    - `al`: 8bits register, lower byte of `ax`
- `b`: same handling as `a` with registers `bh`, `bl` and `bx`
- `c`: same handling as `a` with registers `ch`, `cl` and `cx`
- `d`: same handling as `a` with registers `dh`, `dl` and `dx`
- `ex` : 16bits register
- `fx` : 16bits register
- `gx` : 16bits register
- `hx` : 16bits register

## Instruction

- `mov` `x` `y` (move `x` in `y`):
    - `x` = register, memory (as u8 or u16), literal, register pointer (as u8 or u16)
    - `y` = register, memory (as u8 or u16), register pointer (as u8 or u16)
- `add` `x` `y` (add `x` and `y` in register acc):
    - `x` = register
    - `y` = literal, register
    - `x` + `y` > 65535 -> flag Carry == 0100
- `cmp` `x` `y` (update flag by comparing `x` and `y`):
    - `x` = register
    - `y` = literal, register
    - `x` == `y` -> flag Zero == 0001
    - `x` > `y` -> flag Pos == 0000
    - `x` < `y` -> flag Neg == 0010
- `inc` `reg` (increment value in `reg`)
- `dec` `rec` (decrement value in `reg`)
- `jmp` `label` (jmp to label unconditionnaly):
    - `jmp` = `jne` (jmp if flag Zero is not set)
    - `jmp` = `jeq` (jmp if flag Zero is set)
    - `jmp` = `jgt` (jmp if flag Zero is not set AND flag Neg is not set)
    - `jmp` = `jge` (jmp if flag Neg is not set)
    - `jmp` = `jlt` (jmp if flag Neg is set)
    - `jmp` = `jle` (jmp if flag Neg is set OR if flag Zero is set)
- `psh` `val` (push val on stack):
    - `val` = register, register pointer (as u8 or u16), literal, memory (as u8 or u16)
- `pop` `val` (pop stack in val):
    - `val` = register, register pointer (as u8 or u16), memory (as u8 or u16)
- `cal` `val` (call subroutine, setting up stackframe):
    - `val` = label or register
- `ret` (return from subroutine, restoring stackframe)
- `xor` `x` `y` (xor `x` and `y` and store result in `x`):
    - `x` = register
    - `y` = literal or register
- `end` (end of program)

## Memory map

0x3000-0x4000 -> screen
