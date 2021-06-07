use super::*;
use std::cell::RefCell;
use core::sync::atomic::AtomicUsize;

static THRESHOLD_ADJ: f32 = 1.4;

struct GcStats {
    bytes_allocated: AtomicUsize,
    threshold: AtomicUsize,
}

thread_local!(static STATS: GcStats = GcStats {
    bytes_allocated: AtomicUsize::new(0),
    threshold: AtomicUsize::new(100),
});

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

#[inline]
fn force_collect() {
    STATS.with(|stats| {
        let bytes_collected = HEAP.with(|heap| heap.borrow_mut().collect());
        stats.bytes_allocated.fetch_sub(bytes_collected, Ordering::Relaxed);
        let bytes_allocated = stats.bytes_allocated.load(Ordering::Relaxed);
        stats.threshold.store((bytes_allocated as f32 * THRESHOLD_ADJ) as usize, Ordering::Relaxed);
    })
}

#[inline]
fn should_collect() -> bool {
    STATS.with(|stats| {
        let bytes_allocated = stats.bytes_allocated.load(Ordering::Relaxed);
        let threshold = stats.threshold.load(Ordering::Relaxed);
        bytes_allocated > threshold
    })
}

#[inline]
fn add_bytes<T>() {
    STATS.with(|stats| {
        stats.bytes_allocated.fetch_add(std::mem::size_of::<T>(), Ordering::Relaxed);
    })
}
