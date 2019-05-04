pub type InstructionIndex = usize;
pub type ConstantIndex = usize;
pub type StackIndex = usize;
pub type ChunkIndex = usize;

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

// #[derive(Debug)]
// pub struct Function {
//     name: ConstantIndex|String,
//     chunk_index: ChunkIndex,
//     arity: usize,
//     // upvalues: usize,
// }

#[derive(Debug, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    //Function(Function),
}

impl From<f64> for Constant {
    fn from(item: f64) -> Self { Constant::Number(item) }
}
impl From<&str> for Constant {
    fn from(item: &str) -> Self { Constant::String(String::from(item)) }
}

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
}

pub struct Module {
    chunks: Vec<Chunk>,
    constants: Vec<Constant>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            chunks: vec![],
            constants: vec![],
        }
    }

    pub fn chunk(&self, index: ChunkIndex) -> &Chunk { &self.chunks[index] }
    pub fn chunk_mut(&mut self, index: ChunkIndex) -> &mut Chunk { &mut self.chunks[index] }

    pub fn add_chunk(&mut self) -> ChunkIndex {
        self.chunks.push(Chunk::new());
        self.chunks.len() - 1
    }
    pub fn add_constant(&mut self, constant: Constant) -> ConstantIndex {
        //TODO|HACK this should use module constants, not first chunk
        self.chunk_mut(0).add_constant(constant);
        self.chunk(0).constants.len() - 1
    }
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            instructions: vec![],
            constants: vec![],
        }
    }
    pub fn add_instruction(&mut self, instruction: Instruction) -> InstructionIndex {
        self.instructions.push(instruction);
        self.instructions.len() - 1
    }

    pub fn add_two_instructions(
        &mut self,
        instruction_one: Instruction,
        instruction_two: Instruction,
    ) -> InstructionIndex {
        self.add_instruction(instruction_one);
        self.add_instruction(instruction_two);
        self.instructions.len() - 2
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
