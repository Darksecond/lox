use fxhash::FxHashMap;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Symbol(u32);

impl Symbol {
    pub const fn invalid() -> Self {
        Self(0)
    }
}

pub struct Interner {
    next: u32,
    map: FxHashMap<String, Symbol>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            next: 1,
            map: FxHashMap::default(),
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