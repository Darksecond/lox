pub type InstructionIndex = usize;
pub type ConstantIndex = usize;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Constant(ConstantIndex),
    True,
    False,
    Nil,

    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,

    Not,
    Equal,
    Greater,
    Less,
    // etc
}

#[derive(Debug, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            instructions: vec![],
            constants: vec![],
        }
    }
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_two_instructions(
        &mut self,
        instruction_one: Instruction,
        instruction_two: Instruction,
    ) {
        self.add_instruction(instruction_one);
        self.add_instruction(instruction_two);
    }

    pub fn add_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }
}
