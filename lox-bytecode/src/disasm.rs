
pub fn disassemble_chunk(chunk: &[u8]) {
    let mut index = 0;
    while index < chunk.len() {
        index = disassemble_instruction(chunk, index);
    }
}

pub fn disassemble_instruction(chunk: &[u8], offset: usize) -> usize {
    use crate::opcode::*;

    let opcode = chunk[offset];
    match opcode {
        TRUE => simple_instruction("TRUE", offset),
        CLASS => class_instruction("CLASS", offset, chunk),
        _ => unimplemented!("{} not implemented", opcode),
    }
}

fn simple_instruction(instruction: &str, offset: usize) -> usize {
    println!("{:08X} {}", offset, instruction);
    offset + 1
}

fn class_instruction(instruction: &str, offset: usize, chunk: &[u8]) -> usize {
    let index = chunk[offset+1];
    println!("{:08X} {} {:02X}", offset, instruction, index);
    offset + 2
}
