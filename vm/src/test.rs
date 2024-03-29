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

        assert!(m.get_memory_at_u8(0x40).is_err());
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
    fn test_offset() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_MEM16,  0x3, 0x00, 0x15, 0x00,  // put 0x0300 at 0x1500 in memory
            MOV_LIT_REG, 0x01, 0x00, AX,            // put 0x0100 in AX
            MOV_LITOFF_REG, 0x14, 0x00, AX, BX,     // move value in memory at address [0x1400 + AX] in BX
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ax").unwrap(), 0x0100);
        assert_eq!(cpu.get_register("bx").unwrap(), 0x0300);
        assert_eq!(cpu.get_register("bh").unwrap(), 0x03);
    }

    #[test]
    fn test_offset2() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_MEM16,  0x3, 0x45, 0x15, 0x00,  // put 0x0345 at 0x1500 in memory
            MOV_LIT_REG, 0x01, 0x00, AX,            // put 0x0100 in AX
            MOV_LITOFF_REG, 0x14, 0x00, AX, BH,     // move value in memory at address [0x1400 + AX] in BH but is only a 8bits register so data will be lost
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ax").unwrap(), 0x0100);
        assert_eq!(cpu.get_register("bh").unwrap(), 0x45); // lost upper byte of data -> 0x03
    }

    #[test]
    fn test_subtractions() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x04, AH,
            MOV_LIT_REG, 0x00, 0x03, AL,
            SUB_REG_REG, AL, AH,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ah").unwrap(), 0x04);
        assert_eq!(cpu.get_register("al").unwrap(), 0x03);
        assert_eq!(cpu.get_register("ax").unwrap(), 0x0403);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x0001);
    }

    #[test]
    fn test_subtractions2() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x04, AH,
            SUB_LIT_REG, 0x00, 0x03, AH,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ah").unwrap(), 0x04);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x0001);
    }

    #[test]
    fn test_subtractions3() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x04, AH,
            SUB_REG_LIT, AH, 0x00, 0x05,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ah").unwrap(), 0x04);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x0001);
    }

    #[test]
    fn test_multiplication() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x04, AH,
            MOV_LIT_REG, 0x00, 0x03, AL,
            MUL_REG_REG, AL, AH,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ah").unwrap(), 0x04);
        assert_eq!(cpu.get_register("al").unwrap(), 0x03);
        assert_eq!(cpu.get_register("ax").unwrap(), 0x0403);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x000C);
    }

    #[test]
    fn test_multiplication2() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x04, AH,
            MUL_REG_LIT, AH, 0x00, 0x03,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ah").unwrap(), 0x04);
        assert_eq!(cpu.get_register("acc").unwrap(), 0x000C);
    }

    #[test]
    fn test_shifts() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x00, 0x01, AL,
            MOV_LIT_REG,  0x00, 0x01, BL,
            MOV_LIT_REG,  0x00, 0x02, CL,
            LSF_REG_LIT, AL, 0x00, 0x02,
            RSF_REG_LIT, AL, 0x00, 0x02,
            LSF_REG_REG, BL, CL,
            RSF_REG_REG, BL, CL,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("al").unwrap(), 0x01);
        assert_eq!(cpu.get_register("bl").unwrap(), 0x01);
        assert_eq!(cpu.get_register("cl").unwrap(), 0x02);
    }

    #[test]
    fn test_and_or_xor_not() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG,  0x01, 0x01, AX,
            MOV_LIT_REG,  0x01, 0x01, BX,
            MOV_LIT_REG,  0x01, 0x01, CX,
            MOV_LIT_REG,  0x01, 0x01, DX,
            AND_REG_LIT, AH, 0x00, 0x03,
            OR_REG_LIT, AL, 0x00, 0x03,
            XOR_REG_LIT, BL, 0x00, 0x03,
            NOT, BH,
            AND_REG_REG, CH, AH,
            OR_REG_REG, CL, BL,
            XOR_REG_REG, DH, DL,
            END,
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("ax").unwrap(), 0x0103);
        assert_eq!(cpu.get_register("bx").unwrap(), 0xFE02);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0103);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0001);
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

    #[test]
    fn jump_unconditional() {
        let mut cpu = CPU::default();
        let instructions = [
            MOV_LIT_REG, 0x00, 0x11, AX, // 0x0000
            JMP_LIT, 0x00, 0x0B,         // 0x0004
            MOV_LIT_REG, 0x00, 0x01, BH, // 0x0007
            JMP_REG, AX,                 // 0x000B
            MOV_LIT_REG, 0x00, 0x01, BL, // 0x000D
            END                          // 0x0011
        ];

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0000);
    }

    macro_rules! jump_code {
        ($ins_lit:expr, $ins_reg:expr) => {
            [
                MOV_LIT_REG, 0x00, 0x2F, AX, // 0x0000
                CMP_REG_LIT, AX, 0x00, 0x30, // 0x0004
                $ins_lit, 0x00, 0x0F,        // 0x0008
                MOV_LIT_REG, 0x00, 0x01, BH, // 0x000B
                CMP_REG_LIT, AX, 0x00, 0x2F, // 0x000F
                $ins_lit, 0x00, 0x1A,        // 0x0013
                MOV_LIT_REG, 0x00, 0x01, CH, // 0x0016
                CMP_REG_LIT, AX, 0x00, 0x20, // 0x001A
                $ins_lit, 0x00, 0x25,        // 0x001E
                MOV_LIT_REG, 0x00, 0x01, DH, // 0x0021

                CMP_REG_LIT, AX, 0x00, 0x30, // 0x0025
                $ins_reg, AX,                // 0x0029
                MOV_LIT_REG, 0x00, 0x01, BL, // 0x002B
                MOV_LIT_REG, 0x00, 0x3D, AX, // 0x002F
                CMP_REG_LIT, AX, 0x00, 0x3D, // 0x0033
                $ins_reg, AX,                // 0x0037
                MOV_LIT_REG, 0x00, 0x01, CL, // 0x0039
                MOV_LIT_REG, 0x00, 0x4B, AX, // 0x003D
                CMP_REG_LIT, AX, 0x00, 0x40, // 0x0041
                $ins_reg, AX,                // 0x0045
                MOV_LIT_REG, 0x00, 0x01, DL, // 0x0047
                END                          // 0x004B
            ]
        };
    }

    #[test]
    fn jump_equal() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JEQ_LIT, JEQ_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0101);
    }

    #[test]
    fn jump_not_equal() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JNE_LIT, JNE_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0000);
    }

    #[test]
    fn jump_greater_than() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JGT_LIT, JGT_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0000);
    }

    #[test]
    fn jump_greater_or_equal() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JGE_LIT, JGE_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0000);
    }

    #[test]
    fn jump_lower_than() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JLT_LIT, JLT_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0101);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0101);
    }

    #[test]
    fn jump_lower_or_equal() {
        let mut cpu = CPU::default();
        let instructions = jump_code!(JLE_LIT, JLE_REG);

        cpu.set_instruction(&instructions);
        while cpu.step() {}

        assert_eq!(cpu.get_register("bx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("cx").unwrap(), 0x0000);
        assert_eq!(cpu.get_register("dx").unwrap(), 0x0101);
    }
}
