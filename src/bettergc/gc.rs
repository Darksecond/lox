use super::*;
use std::cell::RefCell;

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
    collect_if_needed();
    add_bytes::<T>();
    HEAP.with(|heap| heap.borrow_mut().manage_root(data))
}

pub fn unique<T: 'static + Trace>(data: T) -> UniqueRoot<T> {
    collect_if_needed();
    add_bytes::<T>();
    HEAP.with(|heap| heap.borrow_mut().unique(data))
}

pub fn root<T: 'static + Trace + ?Sized>(obj: Gc<T>) -> Root<T> {
    HEAP.with(|heap| heap.borrow_mut().root(obj))
}

pub fn force_collect() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.bytes_allocated -= collect();
    })
}

fn collect() -> usize {
    HEAP.with(|heap| heap.borrow_mut().collect())
}

fn collect_if_needed() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        if stats.bytes_allocated > stats.threshold {
            stats.bytes_allocated -= collect();

            stats.threshold = (stats.bytes_allocated as f32 * 1.4) as usize;
        }
        
    })
}

fn add_bytes<T>() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.bytes_allocated += std::mem::size_of::<T>();
    })
}