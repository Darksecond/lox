use super::*;
use std::cell::RefCell;

static THRESHOLD_ADJ: f32 = 1.4;

struct GcStats {
    bytes_allocated: usize,
    threshold: usize,
}

thread_local!(static STATS: RefCell<GcStats> = RefCell::new(GcStats {
    bytes_allocated: 0,
    threshold: 100,
}));

thread_local!(static HEAP: RefCell<Heap> = RefCell::new(Heap::new()));

pub fn manage<T: 'static + Trace>(data: T) -> Root<T> {
    add_bytes::<T>();
    HEAP.with(|heap| heap.borrow_mut().manage(data))
}

pub fn unique<T: 'static + Trace>(data: T) -> UniqueRoot<T> {
    add_bytes::<T>();
    HEAP.with(|heap| heap.borrow_mut().unique(data))
}

pub fn root<T: 'static + Trace + ?Sized>(obj: Gc<T>) -> Root<T> {
    HEAP.with(|heap| heap.borrow_mut().root(obj))
}

pub fn collect() {
    if should_collect() {
        force_collect();
    }
}

fn force_collect() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.bytes_allocated -= HEAP.with(|heap| heap.borrow_mut().collect());
        stats.threshold = (stats.bytes_allocated as f32 * THRESHOLD_ADJ) as usize;
    })
}

fn should_collect() -> bool {
    STATS.with(|stats| {
        let stats = stats.borrow();
        stats.bytes_allocated > stats.threshold
    })
}

fn add_bytes<T>() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.bytes_allocated += std::mem::size_of::<T>();
    })
}
