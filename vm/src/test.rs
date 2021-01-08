#[cfg(test)]
mod tests {
    #[test]
    fn cpu_register_test() {
        use crate::component::cpu::CPU;

        let mut cpu = CPU::new(0x40);

        cpu.set_register("r1", 0x8574).unwrap();
        cpu.set_register("r6", 0x20).unwrap();

        cpu.print_registers();
        assert_eq!(cpu.get_register("r1").unwrap(), 0x8574);
    }

    #[test]
    fn cpu_mov_add_test() {
        use crate::component::cpu::CPU;
        use arch::{
            instructions::*,
            register::{ACC, R1, R2},
        };

        let mut cpu = CPU::new(0x40);

        let instructions = [
            MOV_LIT_REG, 0xFF, 0xFF, R1,  // move 0xFFFF in r1 (16 bit)
            MOV_LIT_REG, 0x00, 0x02, R2,  // move 0x0001 in r2 (16 bit)
            MOV_LIT_REG, 0x00, 0x4F, ACC, // fill ACC with non-null value
            ADD_REG_REG, R1,   R2,        // add r1 and r2
            END,                          // stop the program
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}
        assert_eq!(cpu.get_register("r1").unwrap(), 0xFFFF);
        assert_eq!(cpu.get_register("r2").unwrap(), 0x02);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x01);
    }

    #[test]
    fn cpu_jmp_xor_test() {
        use crate::component::cpu::CPU;
        use arch::{
            instructions::*,
            register::{ACC, R1, R2},
        };

        let mut cpu = CPU::new(0xFF);

        let instructions = [
            MOV_LIT_REG, 0x00, 0x01, R2,         // move 0x0001 in r2 (16 bit)
            MOV_REG_REG, ACC,  R1,               // store accumulator value in memory address 0x0080
            ADD_REG_REG, R1,   R2,               // add r1 and r2
            JMP_NOT_EQ,  0x00, 0x02, 0x00, 0x04, // Jump to address 0x0000 in memory if accumulator not equal to 0x0004
            XOR_REG_REG, R1,   R1,               // XOR register with himself to set to 0
            XOR_REG_REG, R2,   R2,               // XOR register with himself to set to 0
            END,                                 // stop the program
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}
        assert_eq!(cpu.get_register("r1").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("r2").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x02);
    }

    #[test]
    fn memory_test() {
        use crate::component::memory::Memory;
        let mut m = Memory::new(0x40);
        m.set_memory_at_u8(0x01, 0x01).unwrap();
        m.set_memory_at_u8(0x05, 0x20).unwrap();

        assert_eq!(m.get_memory_at_u8(0x04).unwrap(), 0x00);
        assert_eq!(m.get_memory_at_u8(0x01).unwrap(), 0x01);
        assert_eq!(m.get_memory_at_u8(0x05).unwrap(), 0x20);
        assert_eq!(m.get_memory_at_u8(0x40).is_err(), true);
    }

    #[test]
    fn swap_registers_with_stack() {
        use crate::component::cpu::CPU;
        use arch::{
            instructions::*,
            register::{R1, R2},
        };

        let mut cpu = CPU::new(0x20);
        let instructions = [
            MOV_LIT_REG, 0x00, 0x4F, R1,
            MOV_LIT_REG, 0xF4, 0x00, R2,
            PSH_REG,     R1,
            PSH_REG,     R2,
            POP_REG,     R1,
            POP_REG,     R2,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("r1").unwrap(), 0xF400);
        assert_eq!(cpu.get_register("r2").unwrap(), 0x004F);
        assert_eq!(cpu.get_register("sp").unwrap(), 0x001E);
    }
}
