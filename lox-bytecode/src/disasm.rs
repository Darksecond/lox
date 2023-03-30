use crate::opcode::Opcode;
use crate::bytecode::Module;

pub fn disassemble_module(module: &Module) {
    println!("=== Start of Dump ===");
    println!();

    for (index, chunk) in module.chunks().iter().enumerate() {
        println!("=== Chunk {} ===", index);
        disassemble_chunk(chunk.as_slice());
        println!();
    }

    println!("=== Classes ===");
    for (index, class) in module.classes().iter().enumerate() {
        println!("{} {}", index, class.name);
    }
    println!();

    println!("=== Closures ===");
    for (index, closure) in module.closures().iter().enumerate() {
        println!("{} {:?}", index, closure);
    }
    println!();

    println!("=== Identifiers ===");
    for (index, identifier) in module.identifiers().iter().enumerate() {
        println!("{} {}", index, identifier);
    }
    println!();

    println!("=== Constants ===");
    for (index, constant) in module.constants().iter().enumerate() {
        println!("{} {:?}", index, constant);
    }
    println!();

    println!("=== End of Dump ===");
    println!();
}

pub fn disassemble_chunk(chunk: &[u8]) {
    let chunk = crate::opcode::OpcodeIterator::new(chunk.iter().cloned());
    for (offset, opcode) in chunk {
        match opcode {
            Opcode::Jump(relative) => println!("{:04X} {:?} ({:04X})", offset, opcode, absolute(offset, relative)),
            Opcode::JumpIfFalse(relative) => println!("{:04X} {:?} ({:04X})", offset, opcode, absolute(offset, relative)),
            _ => println!("{:04X} {:?}", offset, opcode),
        }
    }
}

fn absolute(offset: usize, relative: i16) -> usize {
    let offset = offset as i64;
    let relative = relative as i64;
    let absolute = offset + relative + 3;
    absolute as usize
}
