#![no_std]
#![no_main]
#![allow(unused_variables)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(allocator_api)]

extern crate alloc;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    memory::init_heap();

    klog::init();

    log::info!("Reduct OS kernel starting...");

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}

pub mod klog;
pub mod memory;
pub mod serial;
