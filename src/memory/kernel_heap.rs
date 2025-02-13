use good_memory_allocator::SpinLockedAllocator;
use x86_64::VirtAddr;

use super::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 8 * 1024 * 1024;

#[global_allocator]
pub static KERNEL_ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

pub fn init_heap() {
    let heap_start = VirtAddr::new(HEAP_START as u64);

    MemoryManager::alloc_range(
        heap_start,
        HEAP_SIZE as u64,
        MappingType::KernelData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    unsafe {
        KERNEL_ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
