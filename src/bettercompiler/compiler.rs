use crate::bytecode::*;
use super::CompilerError;
use super::locals::*;

#[derive(Copy, Clone)]
pub enum ContextType {
    Function,
    Initializer,
    Method,
    TopLevel,
}

struct CompilerContext {
    context_type: ContextType,
    chunk_index: ChunkIndex,
    locals: Locals,
    upvalues: Vec<Upvalue>,
}

pub struct Compiler {
    module: Module,
    contexts: Vec<CompilerContext>,
}

impl CompilerContext {
    fn new(context_type: ContextType, chunk_index: ChunkIndex) -> CompilerContext {
        CompilerContext {
            context_type,
            chunk_index,
            locals: Locals::new(),
            upvalues: vec![],
        }
    }

    fn add_upvalue(&mut self, upvalue: Upvalue) -> StackIndex {
        for i in 0..self.upvalues.len() {
            let existing_upvalue = &self.upvalues[i];
            if upvalue == *existing_upvalue {
                return i;
            }
        }

        self.upvalues.push(upvalue);

        self.upvalues.len() - 1
    }

    fn resolve_local(&self, name: &str) -> Result<Option<StackIndex>, CompilerError> {
        if let Some(local) = self.locals.get(name) {
            if !local.initialized() {
                Err(CompilerError::LocalNotInitialized)
            } else { Ok(Some(local.slot()))}
        } else {
            Ok(None)
        }
    }
}

impl Compiler {
    fn current_context(&self) -> Result<&CompilerContext, CompilerError> {
        self.contexts.last().ok_or(CompilerError::NoContext)
    }
    fn current_context_mut(&mut self) -> Result<&mut CompilerContext, CompilerError> {
        self.contexts.last_mut().ok_or(CompilerError::NoContext)
    }
    fn current_chunk_mut(&mut self) -> Result<&mut Chunk, CompilerError> {
        Ok(self.module.chunk_mut(self.current_context()?.chunk_index))
    }
    fn current_chunk(&self) -> Result<&Chunk, CompilerError> {
        Ok(self.module.chunk(self.current_context()?.chunk_index))
    }
    fn begin_context(&mut self, context_type: ContextType) {
        let chunk = self.module.add_chunk();
        self.contexts.push(CompilerContext::new(context_type, chunk));
    }
    fn end_context(&mut self) -> Result<(ChunkIndex, Vec<Upvalue>), CompilerError> {
        let context = self.contexts.pop().ok_or(CompilerError::NoContext)?;
        Ok((context.chunk_index, context.upvalues))
    }
    fn begin_scope(&mut self) -> Result<(), CompilerError> {
        self.current_context_mut()?.locals.begin_scope();
        Ok(())
    }
    fn end_scope(&mut self) -> Result<(), CompilerError> {
        for local in self.current_context_mut()?.locals.end_scope().iter().rev() {
            if local.captured() {
                self.add_instruction(Instruction::CloseUpvalue)?;
            } else {
                self.add_instruction(Instruction::Pop)?;
            }
        }
        Ok(())
    }

    pub fn new() -> Compiler {
        Compiler {
            module: Module::new(),
            contexts: vec![],
        }
    }

    pub fn into_module(self) -> Module { self.module }

    pub fn context_type(&self) -> Result<ContextType, CompilerError> {
        Ok(self.current_context()?.context_type)
    }

    pub fn with_scope<F>(&mut self, f: F) -> Result<(), CompilerError>  where F: FnOnce(&mut Self) -> Result<(), CompilerError> {
        self.begin_scope()?;
        let result = f(self);
        self.end_scope()?;
        result
    }

    pub fn is_scoped(&mut self) -> bool {
        self.current_context().and_then(|c: &CompilerContext| Ok(c.locals.scope_depth() > 0)).unwrap_or(false)
    }

    pub fn with_context<F>(&mut self, context_type: ContextType, f: F) -> Result<(ChunkIndex, Vec<Upvalue>), CompilerError> where F: FnOnce(&mut Self) -> Result<(), CompilerError> {
        self.begin_context(context_type);
        let result = f(self);
        let ctx_result = self.end_context();
        if result.is_err() && ctx_result.is_err() {
            return Err(CompilerError::Multiple(vec![result.unwrap_err(), ctx_result.unwrap_err()]));
        }
        result?;
        ctx_result
    }

    pub fn with_scoped_context<F>(&mut self, context_type: ContextType, f: F) -> Result<(ChunkIndex, Vec<Upvalue>), CompilerError> where F: FnOnce(&mut Self) -> Result<(), CompilerError> {
        self.with_context(context_type, |compiler| {
            compiler.begin_scope()?;
            f(compiler)
        })
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> Result<InstructionIndex, CompilerError> {
        Ok(self.current_chunk_mut()?.add_instruction(instruction))
    }

    pub fn patch_instruction(&mut self, index: InstructionIndex) -> Result<(), CompilerError> {
        self.current_chunk_mut()?.patch_instruction(index);
        Ok(())
    }

    pub fn patch_instruction_to(&mut self, index: InstructionIndex, to: InstructionIndex) -> Result<(), CompilerError> {
        self.current_chunk_mut()?.patch_instruction_to(index, to);
        Ok(())
    }

    pub fn instruction_index(&self) -> Result<InstructionIndex, CompilerError> {
        Ok(self.current_chunk()?.instruction_index())
    }

    pub fn add_local(&mut self, name: &str) -> Result<StackIndex, CompilerError> {
        self.current_context_mut()?.locals
            .insert(name)
            .map(|l| l.slot())
            .ok_or(CompilerError::LocalAlreadyDefined)
    }

    pub fn mark_local_initialized(&mut self) -> Result<(), CompilerError> { //TODO refactor
        self.current_context_mut()?.locals.mark_initialized();
        Ok(())
    }

    pub fn resolve_local(&self, name: &str) -> Result<Option<StackIndex>, CompilerError> {
        self.current_context()?.resolve_local(name)
    }

    fn resolve_local_in_context(name: &str, context: &CompilerContext) -> Result<Option<StackIndex>, CompilerError> {
        if let Some(local) = context.locals.get(name) {
            if !local.initialized() {
                Err(CompilerError::LocalNotInitialized)
            } else { Ok(Some(local.slot()))}
        } else {
            Ok(None)
        }
    }

    pub fn add_constant<C: Into<Constant>>(&mut self, constant: C) -> ConstantIndex {
        self.module.add_constant(constant.into())
    }

    pub fn resolve_upvalue(&mut self, name: &str) -> Result<Option<StackIndex>, CompilerError> {
        for i in (0..(self.contexts.len()-1)).rev() { // Skip the current context
            if let Some(local) = Self::resolve_local_in_context(name, &self.contexts[i])? {
                self.contexts[i].locals.mark_captured(local);
                let mut upvalue = self.contexts[i+1].add_upvalue(Upvalue::Local(local));
                for j in (i+2)..self.contexts.len() {
                    upvalue = self.contexts[j].add_upvalue(Upvalue::Upvalue(upvalue));
                }
                return Ok(Some(upvalue));
            }
        }

        Ok(None)
    }
}