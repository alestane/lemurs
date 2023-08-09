#![cfg_attr(not(test), no_std)]

extern crate cpp_panic;

use core::alloc::{GlobalAlloc, Layout};

extern {
	fn cpp_allocate(s: usize) -> *mut u8;
	fn cpp_deallocate(w: *mut u8);
}

struct Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

unsafe impl GlobalAlloc for Allocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let size = layout.size() + layout.align() - 1;
		cpp_allocate(size - (size % layout.align()))
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
		cpp_deallocate(ptr)
	}
}