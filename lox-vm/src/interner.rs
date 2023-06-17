use std::{collections::HashMap, cell::RefCell};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Symbol(pub u32);

impl Symbol {
    pub const fn invalid() -> Self {
        Self(0)
    }

    pub fn to_string(self) -> String {
        INTERNER.with(|interner| {
            interner.borrow().reverse_map.get(&self).unwrap().clone()
        })
    }
}

struct Interner {
    next: u32,
    map: HashMap<String, Symbol>,
    reverse_map: HashMap<Symbol, String>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            next: 1,
            map: HashMap::default(),
            reverse_map: HashMap::default(),
        }
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(symbol) = self.map.get(string) {
            *symbol
        } else {
            let symbol = Symbol(self.next);
            self.next += 1;
            self.map.insert(string.to_string(), symbol);
            self.reverse_map.insert(symbol, string.to_string());
            symbol
        }
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

pub fn intern(string: &str) -> Symbol {
    INTERNER.with(|interner| {
        interner.borrow_mut().intern(string)
    })
}
