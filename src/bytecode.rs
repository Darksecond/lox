pub type InstructionIndex = usize;
pub type ConstantIndex = usize;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Constant(ConstantIndex),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
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
            instructions: vec!(),
            constants: vec!(),
        }
    }
    pub fn add_instruction(&mut self, instruction: Instruction) -> InstructionIndex{
        self.instructions.push(instruction);
        self.instructions.len() - 1
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