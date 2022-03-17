use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Symbol(u32);

impl Symbol {
    pub const fn invalid() -> Self {
        Self(0)
    }
}

pub struct Interner {
    next: u32,
    map: HashMap<String, Symbol>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            next: 1,
            map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(symbol) = self.map.get(string) {
            *symbol
        } else {
            let symbol = Symbol(self.next);
            self.next += 1;
            self.map.insert(string.to_string(), symbol);
            symbol
        }
    }
}