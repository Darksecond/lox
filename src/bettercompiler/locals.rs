
#[derive(Debug)]
pub struct Local {
    name: String,
    depth: usize,
    slot: usize,
    initialized: bool,
    is_upvalue: bool,
}

impl Local {
    pub fn slot(&self) -> usize { self.slot }
    pub fn initialized(&self) -> bool { self.initialized }
}

#[derive(Debug)]
pub struct Locals {
    stack: Vec<Local>,
    scope_depth: usize,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            stack: vec![],
            scope_depth: 0,
        }
    }

    pub fn scope_depth(&self) -> usize { self.scope_depth }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) -> Vec<Local> {
        self.scope_depth -= 1;
        let index = self.stack.iter()
            .enumerate()
            .find_map(|(i,l)| if l.depth > self.scope_depth { Some(i) } else { None })
            .unwrap_or(self.stack.len());
        self.stack.split_off(index)
    }

    pub fn get(&self, identifier: &str) -> Option<&Local> {
        self.stack.iter().rev().find(|l| l.name == identifier)
    }

    fn get_at_depth(&self, identifier: &str, depth: usize) -> Option<&Local> {
        self.stack.iter().rev().find(|l| l.depth == depth && l.name == identifier)
    }

    pub fn mark_initialized(&mut self) {
        let index = self.stack.len() - 1;
        self.stack[index].initialized = true;
    }

    pub fn insert(&mut self, identifier: &str) -> Option<&Local> { //TODO Maybe Result<&Local, ()> instead
        if let Some(_) = self.get_at_depth(identifier, self.scope_depth) {
            None
        } else {
            self.stack.push(Local{
                name: identifier.to_string(),
                depth: self.scope_depth,
                slot: self.stack.len(),
                initialized: false,
                is_upvalue: false,
            });
            self.stack.last()
        }
    }
}