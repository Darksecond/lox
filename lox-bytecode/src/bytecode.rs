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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Upvalue>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Upvalue {
    Local(StackIndex),
    Upvalue(UpvalueIndex),
}

//TODO Merge this into Closure, we'll wait until methods are implemented though
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    instructions: Vec<u8>,
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
        unsafe {
            self.chunks.get_unchecked(index)
        }
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

    pub fn add_u8(&mut self, value: u8) -> InstructionIndex {
        self.instructions.push(value);
        self.instructions.len() - 1
    }

    pub fn add_u32(&mut self, value: u32) -> InstructionIndex {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.instructions.push(bytes[i]);
        }

        self.instructions.len() - 4
    }

    pub fn set_u32(&mut self, index: InstructionIndex, value: u32) {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.instructions[index+i] = bytes[i];
        }
    }

    pub fn instruction_index(&self) -> InstructionIndex {
        self.instructions.len()
    }

    //TODO rework this
    pub fn patch_instruction(&mut self, index: InstructionIndex) {
        let current = self.instruction_index();
        self.patch_instruction_to(index, current)
    }

    //TODO rework this
    pub fn patch_instruction_to(&mut self, index: InstructionIndex, to: InstructionIndex) {
        self.set_u32(index, to as _);
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.instructions
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.instructions.as_ptr()
    }

    #[inline(always)]
    pub fn get_u8(&self, pc: usize) -> u8 {
        unsafe { *self.instructions.get_unchecked(pc) }
    }

    #[inline(always)]
    pub fn get_u32(&self, pc: usize) -> u32 {
        let bytes = unsafe { self.instructions.get_unchecked(pc..pc+4) };
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}
