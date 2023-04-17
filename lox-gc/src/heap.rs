use std::cell::Cell;

//TODO Use mmap again
//TODO Impl mmap ourselves
//TODO Merge reams when sweeping
//TODO Figure out dropping

struct MemoryMap {
    size: usize,
    data: *mut u8,
}

impl MemoryMap {
    pub fn new(size: usize) -> Self {
        unsafe {
            let layout = std::alloc::Layout::array::<u8>(size).unwrap().align_to(4096).unwrap();
            let data = std::alloc::alloc(layout);

            Self {
                size,
                data,
            }
        }
    }

    pub fn data(&self) -> *mut u8 {
        self.data
    }
}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::array::<u8>(self.size).unwrap().align_to(4096).unwrap();
        unsafe {
            std::alloc::dealloc(self.data, layout);
        }
    }
}

/// Returns the number of pages needed for `n` bytes (rounding up).
const fn bytes_to_pages(n: usize) -> usize {
  if n % AddrSpace::PAGE_BYTES != 0 {
    n / AddrSpace::PAGE_BYTES + 1
  } else {
    n / AddrSpace::PAGE_BYTES
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct PdIdx(u32);

struct AddrSpace {
    //mem: mmap::MemoryMap,
    mem: MemoryMap,

    used_pds: Cell<u32>,
    reserved_pds: Cell<u32>,
}

impl AddrSpace {
    const PAGE_BYTES: usize = 4096;

    const DATA_BYTES: usize = 4 * 1024 * 1024 * 1024; // 4G
    //const DATA_BYTES: usize = 64 * 1024 * 1024; // 64MB
    //const DATA_BYTES: usize = 32 * 1024 * 1024; // 32MB
    const DATA_PAGES: usize = Self::DATA_BYTES / Self::PAGE_BYTES;

    const PD_PAGES: usize = bytes_to_pages(Self::DATA_PAGES * std::mem::size_of::<PageDescriptor>());
    const BITMAP_PAGES: usize = bytes_to_pages(Self::DATA_PAGES * std::mem::size_of::<Bitmap>());

    const PD_BYTES: usize = Self::PD_PAGES * Self::PAGE_BYTES;
    const BITMAP_BYTES: usize = Self::BITMAP_PAGES * Self::PAGE_BYTES;

    const TOTAL_PAGES: usize = Self::DATA_PAGES + Self::PD_PAGES + Self::BITMAP_PAGES;
    const TOTAL_BYTES: usize = Self::TOTAL_PAGES * Self::PAGE_BYTES;

    const DATA_START: usize = Self::PD_BYTES + Self::BITMAP_BYTES;

    /// Constructs a new [`AddrSpace`]. This will return `None` on error.
    pub fn create() -> Option<Self> {
        //use mmap::*;

        //let mem = MemoryMap::new(Self::TOTAL_BYTES, &[MapOption::MapReadable, MapOption::MapWritable]).ok()?;
        let mem = self::MemoryMap::new(Self::TOTAL_BYTES);

        Some(Self {
            mem,
            used_pds: Cell::new(0),
            reserved_pds: Cell::new(0),
        })
    }

    /// Constructs a [`PdRef`] for the specified [`PdIdx`].
    pub unsafe fn pd(&self, idx: PdIdx) -> PdRef {
        PdRef {
            idx,
            space: self,
        }
    }

    /// Constructs a [`PdRef`] for the specified index.
    pub unsafe fn pd_at(&self, index: u32) -> PdRef {
        PdRef {
            idx: PdIdx(index),
            space: self,
        }
    }

    /// Constructs a [`PdRef`] for the page `ptr` is in.
    pub unsafe fn pd_of(&self, ptr: *const u8) -> PdRef {
        let base = self.mem.data().add(Self::DATA_START);
        let offset = ptr.offset_from(base as _) as usize;
        let index = offset / Self::PAGE_BYTES;
        self.pd_at(index as _)
    }

    /// Return an iterator over all created [`PageDescriptor`]s.
    pub fn pds(&self) -> impl Iterator<Item = PdRef> {
        let end = self.used_pds.get();
        let mut index = self.reserved_pds.get();
        std::iter::from_fn(move || {
            if index >= end {
                return None;
            }

            let pd = unsafe {
                self.pd_at(index)
            };

            index += pd.pages() as u32;

            Some(pd)
        })
    }

    pub fn available_pages(&self) -> u32 {
        let used_pages = self.used_pds.get();
        Self::DATA_PAGES as u32 - used_pages
    }

    pub(crate) fn new_reserved(&self) -> PdList {
        debug_assert_eq!(self.reserved_pds.get(), self.used_pds.get(), "You cannot call new_reserved after init");

        let ream = self.new_ream(1);
        self.reserved_pds.set(self.reserved_pds.get() + 1);
        PdList::new(ream.idx)
    }

    /// Constructs a ream of specified size.
    pub fn new_ream(&self, count: u32) -> PdRef {
        let used_pages = self.used_pds.get();

        debug_assert!(count > 0);
        debug_assert!((used_pages + count) as usize <= Self::DATA_PAGES);

        self.used_pds.set(used_pages+count);

        unsafe {
            let ptr = self.mem.data().cast::<PageDescriptor>().add(used_pages as _);
            ptr.write_bytes(0, count as _);

            let ptr = self.mem.data().add(Self::PD_BYTES).cast::<Bitmap>().add(used_pages as _);
            ptr.write_bytes(0, count as _);
        }

        let page = unsafe {
            self.pd_at(used_pages)
        };

        unsafe { page.force_unlink() };
        page.pd().len.set(count-1);
        page
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
enum SizeClass {
    Ream = 0,
    Block16 = 4,
    Block32 = 5,
    Block64 = 6,
    Block128 = 7,
    Block256 = 8,
    Block512 = 9,
    Block1024 = 10,
    Block2048 = 11,
    Block4096 = 12,
}

impl SizeClass {
    const SMALL: [Self; 9] = [
        Self::Block16,
        Self::Block32,
        Self::Block64,
        Self::Block128,
        Self::Block256,
        Self::Block512,
        Self::Block1024,
        Self::Block2048,
        Self::Block4096,
    ];

    pub const fn block_bytes(self) -> Option<usize> {
        if let Self::Ream = self {
            return None;
        }

        Some(1 << (self as usize))
    }

    pub const fn total_blocks(self) -> usize {
        if let Some(size) = self.block_bytes() {
            AddrSpace::PAGE_BYTES / size
        } else {
            1
        }
    }
}

#[derive(Debug)]
struct PageDescriptor {
    prev: Cell<PdIdx>,
    next: Cell<PdIdx>,

    class: Cell<SizeClass>,

    /// Additional pages
    len: Cell<u32>,
}

#[derive(Copy, Clone)]
struct PdRef<'space> {
    idx: PdIdx,
    space: &'space AddrSpace,
}

impl<'space> PdRef<'space> {
    fn pd(self) -> &'space PageDescriptor {
        unsafe {
            let offset = self.idx.0 as usize * std::mem::size_of::<PageDescriptor>();
            let ptr = self.space.mem.data().add(offset).cast::<PageDescriptor>();
            &*ptr
        }
    }

    fn bitmap(self) -> &'space Bitmap {
        unsafe {
            let offset = self.idx.0 as usize * std::mem::size_of::<Bitmap>() + AddrSpace::PD_BYTES;
            let ptr = self.space.mem.data().add(offset).cast::<Bitmap>();
            &*ptr
        }
    }

    pub fn pages(self) -> usize {
        self.pd().len.get() as usize + 1
    }

    pub fn is_single_page(self) -> bool {
        self.pages() == 1
    }

    pub fn set_class(self, size_class: SizeClass) {
        self.pd().class.set(size_class)
    }

    pub fn class(self) -> SizeClass {
        self.pd().class.get()
    }

    pub fn is_linked(self) -> bool {
        self.pd().prev.get() != self.idx || self.pd().next.get() != self.idx
    }

    //TODO Move to bitmap
    pub fn set_empty(self) {
        for word in self.bitmap().0.iter() {
            word.set(0)
        }
    }

    //TODO Move to bitmap
    pub fn is_empty(self) -> bool {
        self.bitmap().0.iter().all(|w| w.get() == 0)
    }

    //TODO Move to bitmap
    pub fn is_full(self) -> bool {
        let mut blocks = self.class().total_blocks();
        let mut mask = || {
            let mask = if blocks >= 64 {
                !0
            } else {
                (1u64 << blocks) - 1
            };

            blocks = blocks.saturating_sub(64);
            mask
        };

        let mut is_full = true;

        for word in self.bitmap().0.iter() {
            is_full &= word.get() == mask();
        }

        is_full
    }

    fn bits_used(self) -> u32 {
        let mut total = 0;
        for word in self.bitmap().0.iter() {
            total += word.get().count_ones();
        }
        total
    }

    fn bytes_used(self) -> u32 {
        let class = self.class();
        if class == SizeClass::Ream {
            if self.is_empty() {
                0
            } else {
                (self.pages() * AddrSpace::PAGE_BYTES) as u32
            }
        } else {
            self.bits_used() * class.block_bytes().unwrap() as u32
        }
    }

    pub fn data(self, index: usize) -> *mut u8 {
        let offset = AddrSpace::DATA_START
            + self.idx.0 as usize * AddrSpace::PAGE_BYTES
            + index * self.class().block_bytes().unwrap_or(1);

        unsafe {
            self.space.mem.data().add(offset)
        }
    }

    pub fn mark(self, index: usize) {
        debug_assert!(index < self.class().total_blocks());

        let word = index / 64;
        let bit = index % 64;

        let word = &self.bitmap().0[word];
        word.set(word.get() | (1 << bit));
    }

    pub fn is_marked(self, index: usize) -> bool {
        debug_assert!(index < self.class().total_blocks());

        let word = index / 64;
        let bit = index % 64;

        let word = &self.bitmap().0[word];
        word.get() & (1 << bit) != 0
    }

    pub fn next(self) -> PdRef<'space> {
        PdRef {
            idx: self.pd().next.get(),
            space: self.space,
        }
    }

    pub fn prev(self) -> PdRef<'space> {
        PdRef {
            idx: self.pd().prev.get(),
            space: self.space,
        }
    }

    /// Force self to be unlinked, this will *not* remove it from any list.
    pub unsafe fn force_unlink(self) {
        self.pd().next.set(self.idx);
        self.pd().prev.set(self.idx);
    }

    pub fn unlink(self) {
        let prev = self.prev();
        let next = self.next();

        prev.pd().next.set(next.idx);
        next.pd().prev.set(prev.idx);

        // Unlink by setting next and prev on self to self.
        self.pd().next.set(self.idx);
        self.pd().prev.set(self.idx);
    }

    //TODO Move to bitmap
    pub fn take_next_block(self) -> Option<usize> {
        let mut blocks = self.class().total_blocks();
        let mut mask = || {
            let mask = if blocks >= 64 {
                !0
            } else {
                (1u64 << blocks) - 1
            };

            blocks = blocks.saturating_sub(64);
            mask
        };

        for (index, word) in self.bitmap().0.iter().enumerate() {
            if word.get() == mask() {
                continue;
            }

            let idx = word.get().trailing_ones() as usize;
            word.set(word.get() | 1 << idx);

            return Some(index*64 + idx);
        }

        None
    }

    pub fn split(self, split: usize) -> Option<(Self, Option<Self>)> {
        assert!(!self.is_single_page());
        assert!(split > 0);

        if self.pages() == split {
            Some((self, None))
        } else if self.pages() < split {
            None
        } else {
            let rest_len = self.pages() - split - 1;
            let rest = unsafe {
                self.space.pd_at(self.idx.0 + split as u32)
            };
            unsafe { rest.force_unlink() };
            rest.pd().len.set(rest_len as _);

            self.pd().len.set(split as u32 - 1);

            Some((self, Some(rest)))
        }
    }
}

struct Bitmap([Cell<u64>; 4]);

struct PdList {
    root: PdIdx,
}

impl PdList {
    pub fn new(root: PdIdx) -> Self {
        Self {
            root,
        }
    }

    pub fn iter<'space>(&self, space: &'space AddrSpace) -> impl Iterator<Item = PdRef<'space>> {
        let root = unsafe {
            space.pd(self.root)
        };

        let mut current = root.next();

        std::iter::from_fn(move || {
            let value = if current.idx == root.idx {
                return None;
            } else {
                current
            };

            current = current.next();

            Some(value)
        })
    }

    pub fn first<'space>(&self, space: &'space AddrSpace) -> Option<PdRef<'space>> {
        let root = unsafe {
            space.pd(self.root)
        };

        if root.next().idx != root.idx {
            Some(root.next())
        } else {
            None
        }
    }

    pub fn push(&self, item: PdRef) {
        let root = unsafe {
            item.space.pd(self.root)
        };

        let next = root.next();

        next.pd().prev.set(item.idx);
        root.pd().next.set(item.idx);

        item.pd().prev.set(self.root);
        item.pd().next.set(next.idx);
    }
}

pub struct Heap {
    space: AddrSpace,

    free_reams: PdList,
    free_pages: PdList,
    free_sized: [PdList; SizeClass::SMALL.len()],

    full_reams: PdList,
    full_pages: PdList,

    bytes_used: Cell<usize>,
}

impl Heap {
    pub fn new() -> Option<Heap> {
        let space = AddrSpace::create()?;

        let heap = Self {
            free_reams: space.new_reserved(),
            free_pages: space.new_reserved(),
            free_sized: [
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
                space.new_reserved(),
            ],

            full_reams: space.new_reserved(),
            full_pages: space.new_reserved(),
            space,
            bytes_used: Cell::new(0),
        };

        let ream = heap.space.new_ream(heap.space.available_pages());
        heap.free_reams.push(ream);

        Some(heap)
    }

    pub fn bytes_used(&self) -> usize {
        self.bytes_used.get()
    }

    pub fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let bytes = layout.size();
        if bytes <= AddrSpace::PAGE_BYTES {
            let bin_log = bytes.next_power_of_two().trailing_zeros();
            let class = SizeClass::SMALL[bin_log.saturating_sub(4) as usize]; //TODO constant
            self.alloc_small(class)
        } else {
            self.alloc_ream(bytes_to_pages(bytes) as _)
        }
    }

    pub unsafe fn start_gc(&self) {
        self.space.pds()
            .for_each(|pd| pd.set_empty());
    }

    pub unsafe fn mark(&self, ptr: *const u8) {
        let page = self.space.pd_of(ptr);
        let offset = ptr.offset_from(page.data(0)) as usize;
        let index = offset / page.class().block_bytes().unwrap_or(1);

        page.mark(index);
    }

    pub fn is_marked(&self, ptr: *const u8) -> bool {
        unsafe {
            let page = self.space.pd_of(ptr);
            let offset = ptr.offset_from(page.data(0)) as usize;
            let index = offset / page.class().block_bytes().unwrap_or(1);

            page.is_marked(index)
        }
    }

    pub unsafe fn sweep(&self) {
        let mut count = 0;
        for pd in self.space.pds() {
            count += pd.bytes_used() as usize;

            if pd.is_empty() {
                pd.unlink();
                if pd.is_single_page() {
                    self.free_pages.push(pd);
                } else {
                    self.free_reams.push(pd);
                }
            } else if pd.is_full() {
                // Do nothing
            } else {
                debug_assert!(pd.class() != SizeClass::Ream);
                pd.unlink();
                self.sized_list(pd.class()).push(pd);
            }
        }
        self.bytes_used.set(count);
    }

    fn sized_list(&self, size_class: SizeClass) -> &PdList {
        &self.free_sized[size_class as usize - 4] //TODO constant
    }

    fn take_ream(&self, pages: u32) -> PdRef {
        #[inline(never)]
        #[cold]
        fn alloc_failed(pages: u32) -> ! {
            panic!("No ream of sufficient size available, requested {} pages", pages);
        }

        for ream in self.free_reams.iter(&self.space) {
            let (ream, rest) = match ream.split(pages as _) {
                Some(x) => x,
                None => continue,
            };

            match rest {
                Some(r) if r.is_single_page() => {
                    self.free_pages.push(r);
                },
                Some(r) => {
                    self.free_reams.push(r);
                },
                None => (),
            };

            ream.unlink();
            return ream;
        }

        alloc_failed(pages);
    }

    fn alloc_ream(&self, pages: u32) -> *mut u8 {
        self.bytes_used.set(self.bytes_used.get() + pages as usize * AddrSpace::PAGE_BYTES);
        let page = self.take_ream(pages);

        page.set_class(SizeClass::Ream);
        page.take_next_block();
        self.full_reams.push(page);

        page.data(0)
    }

    fn alloc_small(&self, size_class: SizeClass) -> *mut u8 {
        self.bytes_used.set(self.bytes_used.get() + size_class.block_bytes().unwrap());

        let list = self.sized_list(size_class);

        let page = match list.first(&self.space) {
            Some(page) => page,
            None => {
                let page = match self.free_pages.first(&self.space) {
                    Some(page) => {
                        page.unlink();
                        page
                    },
                    None => self.take_ream(1),
                };

                list.push(page);
                page.set_class(size_class);
                page.set_empty();
                page
            },
        };

        let index = page.take_next_block().expect("Full page in free list");

        if page.is_full() {
            page.unlink();
            self.full_pages.push(page);
        }

        page.data(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn it_works() {
        let heap = Heap::new().unwrap();

        let layout = std::alloc::Layout::new::<u64>();
        let _ptr = heap.alloc(layout) as *mut u64;
        let ptr = heap.alloc(layout) as *mut u64;
        assert_eq!(heap.is_marked(ptr as *const u8), true);
        unsafe {
            ptr.write(1234);
            assert_eq!(ptr.read(), 1234);
        };

        unsafe { heap.start_gc(); }
        assert_eq!(heap.is_marked(ptr as *const u8), false);

        unsafe {
            heap.mark(ptr as *const u8);
            heap.sweep();
        }
        assert_eq!(heap.is_marked(ptr as *const u8), true);
        assert_eq!(heap.is_marked(_ptr as *const u8), false);

        let _ptr2 = heap.alloc(layout) as *mut u64;
        assert!(_ptr == _ptr2);
    }
}
