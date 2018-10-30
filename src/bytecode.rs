pub type InstructionIndex = usize;
pub type ConstantIndex = usize;

pub enum Instruction {
    Constant(ConstantIndex),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    // etc
}

pub enum Constant {
    Number(f64),
    String(String),
}

pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
}

impl Chunk {
    pub fn add_instruction(&mut self, instruction: Instruction) -> InstructionIndex{
        self.instructions.push(instruction);
        self.instructions.len() - 1
    }

    pub fn add_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.push(constant);
        self.constants.len() - 1
    }
}