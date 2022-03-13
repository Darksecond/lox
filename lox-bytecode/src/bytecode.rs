use serde::{Deserialize, Serialize};

pub type InstructionIndex = usize;
pub type ConstantIndex = usize;
pub type StackIndex = usize;
pub type ChunkIndex = usize;
pub type ArgumentCount = usize;
pub type UpvalueIndex = usize;
pub type ClosureIndex = usize;
pub type ClassIndex = usize;
pub type IdentifierIndex = usize;

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
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

    DefineGlobal(IdentifierIndex),
    GetGlobal(IdentifierIndex),
    SetGlobal(IdentifierIndex),
    GetLocal(StackIndex),
    SetLocal(StackIndex),
    GetUpvalue(StackIndex),
    SetUpvalue(StackIndex),
    SetProperty(IdentifierIndex),
    GetProperty(IdentifierIndex),

    Jump(InstructionIndex),
    JumpIfFalse(InstructionIndex),
    Call(ArgumentCount),
    Invoke(IdentifierIndex, ArgumentCount),
    CloseUpvalue,

    Class(ClassIndex),
    Closure(ClosureIndex),
    Method(IdentifierIndex),

    Import(ConstantIndex),
    ImportGlobal(IdentifierIndex),
    // etc
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Upvalue>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Upvalue {
    Local(StackIndex),
    Upvalue(UpvalueIndex),
}

//TODO Merge this into Closure, we'll wait until methods are implemented though
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub chunk_index: ChunkIndex,
    pub arity: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Constant {
    Number(f64),
    String(String),
}

impl From<f64> for Constant {
    fn from(item: f64) -> Self {
        Constant::Number(item)
    }
}
impl From<&str> for Constant {
    fn from(item: &str) -> Self {
        Constant::String(String::from(item))
    }
}

impl From<Function> for Closure {
    fn from(item: Function) -> Self {
        Closure {
            function: item,
            upvalues: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    instructions: Vec<Instruction>,
}

#[derive(Serialize, Deserialize)]
pub struct Module {
    chunks: Vec<Chunk>,
    constants: Vec<Constant>,
    closures: Vec<Closure>,
    classes: Vec<Class>,
    identifiers: Vec<String>,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module").finish()
    }
}

impl Module {
    pub fn new() -> Module {
        Module {
            chunks: vec![],
            constants: vec![],
            closures: vec![],
            classes: vec![],
            identifiers: vec![],
        }
    }

    #[inline]
    pub fn chunk(&self, index: ChunkIndex) -> &Chunk {
        &self.chunks[index]
    }

    pub fn chunk_mut(&mut self, index: ChunkIndex) -> &mut Chunk {
        &mut self.chunks[index]
    }

    pub fn add_chunk(&mut self) -> ChunkIndex {
        self.chunks.push(Chunk::new());
        self.chunks.len() - 1
    }

    pub fn add_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn add_closure(&mut self, closure: Closure) -> ClosureIndex {
        self.closures.push(closure);
        self.closures.len() - 1
    }

    pub fn add_class(&mut self, class: Class) -> ClassIndex {
        self.classes.push(class);
        self.classes.len() - 1
    }

    pub fn add_identifier(&mut self, identifier: &str) -> IdentifierIndex {
        self.identifiers.push(identifier.to_string());
        self.identifiers.len() - 1
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn closures(&self) -> &[Closure] {
        &self.closures
    }

    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    pub fn identifiers(&self) -> &[String] {
        &self.identifiers
    }

    #[inline]
    pub fn constant(&self, index: ConstantIndex) -> &Constant {
        &self.constants[index]
    }

    #[inline]
    pub fn closure(&self, index: ClosureIndex) -> &Closure {
        &self.closures[index]
    }

    #[inline]
    pub fn class(&self, index: ClassIndex) -> &Class {
        &self.classes[index]
    }

    #[inline]
    pub fn identifier(&self, index: IdentifierIndex) -> &str {
        &self.identifiers[index]
    }

    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            instructions: vec![],
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

    #[inline]
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    #[inline]
    pub fn instruction(&self, pc: usize) -> Instruction {
        // self.instructions[pc]
        unsafe { *self.instructions.get_unchecked(pc) }
    }
}
