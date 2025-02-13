use good_memory_allocator::SpinLockedAllocator;
use x86_64::{PhysAddr, VirtAddr};

use super::{KERNEL_PAGE_TABLE, MappingType, MemoryManager, convert_physical_to_virtual};

pub const HEAP_START: usize = 0xFFFF_A000_0000_0000;
pub const HEAP_SIZE: usize = 128 * 1024 * 1024;

#[global_allocator]
pub static KERNEL_ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

pub fn init_heap() {
    let page_table_addr =
        convert_physical_to_virtual(PhysAddr::new(unsafe { x86::controlregs::cr3() }));
    for i in 0..2048 {
        unsafe { *((page_table_addr + i).as_mut_ptr::<u8>()) = 0 };
    }

    let heap_start = VirtAddr::new(HEAP_START as u64);

    MemoryManager::alloc_range(
        heap_start,
        HEAP_SIZE as u64,
        MappingType::UserData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    unsafe {
        KERNEL_ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
