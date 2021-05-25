#[cfg(test)]
mod tests {
    use crate::component::cpu::CPU;
    use arch::{instructions::*, registers::*};

    #[test]
    fn cpu_register_test() {
        let mut cpu = CPU::default();

        cpu.set_register("ax", 0x0102).unwrap();
        cpu.set_register("bh", 0x03).unwrap();
        cpu.set_register("bl", 0x04).unwrap();

        assert_eq!(cpu.get_register("ah").unwrap(), 0x01);
        assert_eq!(cpu.get_register("al").unwrap(), 0x02);
        assert_eq!(cpu.get_register("bx").unwrap(), 0x0304);
    }

    #[test]
    fn cpu_acc_test() {
        let mut cpu = CPU::default();

        let instructions = [
            // basic addition
            MOV_LIT_REG, 0x00, 0x10, AX,  // move 0x10 in r1 (16 bit)
            MOV_LIT_REG, 0x00, 0x0A, BX,  // move 0x0A in r2 (16 bit)
            ADD_REG_REG, AX,   BX,        // add r1 and r2

            // overfolwing addition
            MOV_LIT_REG, 0xFF, 0xFF, AX,  // move 0x10 in r1 (16 bit)
            MOV_LIT_REG, 0x00, 0x10, BX,  // move 0x0A in r2 (16 bit)
            ADD_REG_REG, AX,   BX,        // add r1 and r2
        ];
        let expected = [0x001A, 0x000F];
        cpu.set_instruction(&instructions);

        for expected_val in &expected {
            for _ in 0..3 { cpu.step(); }

            let acc = cpu.get_register("acc").unwrap();
            assert_eq!(*expected_val, acc);
        }
    }

    #[test]
    fn cpu_jmp_xor_test() {
        let mut cpu = CPU::default();

        let instructions = [
            MOV_LIT_REG, 0x00, 0x01, BX,         // move 0x0001 in r2 (16 bit)
            MOV_REG_REG, ACC,  AX,               // store accumulator value in memory address 0x0080
            ADD_REG_REG, AX,   BX,               // add r1 and r2
            CMP_REG_LIT, ACC,  0x00, 0x03,       // compare acc and literal 2
            JNE_LIT,     0x00, 0x04,             // Jump to address 0x0000 in memory if accumulator not equal to 0x0004
            XOR_REG_REG, BX,   BX,               // XOR register with himself to set to 0
            END,                                 // stop the program
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}
        assert_eq!(cpu.get_register("ax").unwrap(), 0x0002);
        assert_eq!(cpu.get_register("bx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x003);
    }

    #[test]
    fn memory_test() {
        use crate::component::memory_io::MemoryIO;
        use crate::component::memory::Memory;

        let mut m = Memory::new(0x40);
        m.set_memory_at_u8(0x01, 0x01).unwrap();
        m.set_memory_at_u8(0x05, 0x20).unwrap();

        assert_eq!(m.get_memory_at_u8(0x04).unwrap(), 0x00);
        assert_eq!(m.get_memory_at_u16(0x04).unwrap(), 0x0020);

        assert_eq!(m.get_memory_at_u8(0x01).unwrap(), 0x01);
        assert_eq!(m.get_memory_at_u16(0x01).unwrap(), 0x0100);

        assert_eq!(m.get_memory_at_u8(0x05).unwrap(), 0x20);
        assert_eq!(m.get_memory_at_u16(0x05).unwrap(), 0x2000);

        assert_eq!(m.get_memory_at_u8(0x40).is_err(), true);
    }

    #[test]
    fn swap_registers_with_stack() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG, 0x00, 0x4F, AX,
            MOV_LIT_REG, 0xF4, 0x00, BX,
            PSH_REG,     AX,
            PSH_REG,     BX,
            POP_REG,     AX,
            POP_REG,     BX,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ax").unwrap(), 0xF400);
        assert_eq!(cpu.get_register("bx").unwrap(), 0x004F);
        assert_eq!(cpu.get_register("sp").unwrap(), 0xFFFE);
    }

    #[test]
    fn call_subroutine() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG, 0x11, 0x11, AX, // 0x0000
            MOV_LIT_REG, 0x33, 0x33, CX, // 0x0004
            PSH_LIT, 0x22, 0x22,         // 0x0008
            CALL_LIT, 0x00, 0x18,        // 0x000B
            POP_REG, BX,                 // 0x000E
            END,                         // 0x0010
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, // next octet is 0x0018
            PSH_LIT, 0xAB, 0xCD,         // 0x0018
            PSH_LIT, 0x12, 0x34,         // 0x001B
            MOV_LIT_REG, 0xFF, 0xFF, BX, // 0x001E
            MOV_LIT_REG, 0xFF, 0xFF, CX, // 0x0022
            RET,                         // 0x0026
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ax").unwrap(), 0x1111);
        assert_eq!(cpu.get_register("bx").unwrap(), 0x2222);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x3333);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0000);
    }
}
