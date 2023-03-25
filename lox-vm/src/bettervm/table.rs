use super::memory::Value;
use super::interner::Symbol;
use crate::bettergc::Trace;

#[derive(Copy, Clone)]
struct Entry {
    key: Symbol,
    value: Value,
}

impl Trace for Entry {
    fn trace(&self) {
        self.value.trace();
    }
}

pub struct Table {
    count: usize,
    capacity: usize,
    max_capacity: usize,
    entries: Box<[Entry]>,
}

impl Trace for Table {
    fn trace(&self) {
        for entry in self.entries.iter() {
            entry.trace();
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    const MAX_LOAD: f32 = 0.75;
    const INITIAL_CAPACITY: usize = 8; // Needs to be a power of two.

    #[allow(dead_code)]
    pub fn new() -> Self {
        let entries = vec![Entry { key: Symbol::invalid(), value: Value::Nil }; Self::INITIAL_CAPACITY].into_boxed_slice();

        Self {
            count: 0,
            capacity: Self::INITIAL_CAPACITY,
            max_capacity: (Self::INITIAL_CAPACITY as f32 * Self::MAX_LOAD) as usize,
            entries,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn has(&self, key: Symbol) -> bool {
        if self.count == 0 {
            return false;
        }

        let index = find_entry(self.capacity, &self.entries, key);
        let entry = &self.entries[index];

        entry.key != Symbol::invalid()
    }

    pub fn set(&mut self, key: Symbol, value: Value) -> bool {
        if self.count + 1 > self.max_capacity {
            self.adjust_capacity();
        }

        let index = find_entry(self.capacity, &self.entries, key);
        let entry = &mut self.entries[index];

        let is_new = entry.key == Symbol::invalid();

        if is_new {
            self.count += 1;
        }

        entry.key = key;
        entry.value = value;

        is_new
    }

    pub fn get(&self, key: Symbol) -> Option<Value> {
        if self.count == 0 {
            return None;
        }

        let index = find_entry(self.capacity, &self.entries, key);
        let entry = &self.entries[index];

        if entry.key == Symbol::invalid() {
            return None;
        }

        Some(entry.value)
    }

    fn adjust_capacity(&mut self) {
        let new_capacity = if self.capacity < 8 { 8 } else { self.capacity * 2 };

        let mut new_entries = vec![Entry { key: Symbol::invalid(), value: Value::Nil }; new_capacity].into_boxed_slice();

        for index in 0..self.capacity {
            let entry = &self.entries[index];
            let new_index = find_entry(new_capacity, &new_entries, entry.key);
            let new_entry = &mut new_entries[new_index];

            new_entry.key = entry.key;
            new_entry.value = entry.value;
        }

        self.max_capacity = (new_capacity as f32 * Self::MAX_LOAD) as usize;
        self.capacity = new_capacity;
        self.entries = new_entries;
    }
}

fn find_entry(capacity: usize, entries: &[Entry], key: Symbol) -> usize {
    let mut index = key.0 as usize & (capacity - 1);

    loop {
        let entry = &entries[index];

        if entry.key == key || entry.key == Symbol::invalid() {
            return index;
        }

        index = (index + 1) & (capacity - 1);
    }
}
