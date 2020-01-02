pub type InstructionIndex = usize;
pub type ConstantIndex = usize;
pub type StackIndex = usize;
pub type ChunkIndex = usize;
pub type ArgumentCount = usize;

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

    Jump(InstructionIndex),
    JumpIfFalse(InstructionIndex),
    Call(ArgumentCount),
    // etc
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk_index: ChunkIndex,
    pub arity: usize,
    // pub upvalues: usize,
}

#[derive(Debug, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Function(Function),
}

impl From<f64> for Constant {
    fn from(item: f64) -> Self { Constant::Number(item) }
}
impl From<&str> for Constant {
    fn from(item: &str) -> Self { Constant::String(String::from(item)) }
}
impl From<Function> for Constant {
    fn from(item: Function) -> Self { Constant::Function(item) }
}

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
}

pub struct Module {
    chunks: Vec<Chunk>,
    // constants: Vec<Constant>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            chunks: vec![],
            // constants: vec![],
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

    pub fn constants(&self) -> &[Constant] {
        &self.chunk(0).constants
    }

    pub fn constant(&self, index: ConstantIndex) -> &Constant {
        &self.chunk(0).constants[index]
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

    pub fn instruction_index(&self) -> InstructionIndex {
        self.instructions.len()
    }

    pub fn patch_instruction(&mut self, index: InstructionIndex) {
        let current = self.instruction_index();
        self.patch_instruction_to(index, current)
    }

    pub fn patch_instruction_to(&mut self, index: InstructionIndex, to: InstructionIndex) {
        match self.instructions[index] {
            Instruction::JumpIfFalse(ref mut placeholder) => *placeholder = to,
            Instruction::Jump(ref mut placeholder) => *placeholder = to,
            _ => (), // Nothing to patch
        };
    }

    fn add_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    //TODO Get this of this method, right now it's used a lot in the VM, which needs a (complete) rewrite
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    fn constant(&self, index: ConstantIndex) -> &Constant {
        &self.constants[index]
    }
}
