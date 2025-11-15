#![no_std]

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    b_count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            b_count: 0,
        }
    }

    fn check_space(&self, required_size: usize, is_byte_alloc: bool) -> bool {
        if is_byte_alloc {
            self.b_pos.saturating_add(required_size) <= self.p_pos
        } else {
            self.p_pos.saturating_sub(required_size) >= self.b_pos
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start.checked_add(size).unwrap_or(0);
        self.b_pos = self.start;
        self.p_pos = self.end;
        self.b_count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let aligned_b_pos = (self.b_pos + layout.align() - 1) & !(layout.align() - 1);
        let new_b_pos = match aligned_b_pos.checked_add(layout.size()) {
            Some(b) => b,
            None => return Err(AllocError::NoMemory),
        };

        if new_b_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = new_b_pos;
        self.b_count += 1;
        Ok(unsafe {NonNull::new_unchecked(aligned_b_pos as *mut u8)})
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let ptr = pos.as_ptr() as usize;
        if ptr + layout.size() == self.b_pos {
            self.b_pos = ptr;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn used_bytes(&self) -> usize {
        let bytes_used = self.b_pos.saturating_sub(self.start);
        let pages_used = self.end.saturating_sub(self.p_pos);
        bytes_used + pages_used
    }

    fn available_bytes(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos)
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let size = num_pages.checked_mul(Self::PAGE_SIZE).ok_or(AllocError::NoMemory)?;
        let unaligned_p_pos = self.p_pos.saturating_sub(size);
        let align_mask = align_pow2 - 1;
        let aligned_new_p_pos = unaligned_p_pos & !align_mask;

        if aligned_new_p_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = aligned_new_p_pos;
        Ok(self.p_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {}

    fn total_pages(&self) -> usize {
        self.total_bytes() / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.end.saturating_sub(self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        self.available_bytes() / Self::PAGE_SIZE
    }
}