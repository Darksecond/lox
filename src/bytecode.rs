pub type InstructionIndex = usize;
pub type ConstantIndex = usize;
pub type StackIndex = usize;

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

    Pop,
    
    Return,
    Print,

    DefineGlobal(ConstantIndex),
    GetGlobal(ConstantIndex),
    SetGlobal(ConstantIndex),
    GetLocal(StackIndex),
    SetLocal(StackIndex),
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

    pub fn add_str_constant(&mut self, constant: &str) -> ConstantIndex {
        self.add_constant(Constant::String(constant.to_string()))
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }
}
