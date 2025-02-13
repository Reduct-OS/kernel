#![no_std]
#![no_main]
#![allow(internal_features)]
#![allow(unused_variables)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![feature(inherent_str_constructors)]
#![feature(let_chains)]
#![feature(naked_functions)]

use smp::BSP_LAPIC_ID;

extern crate alloc;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    memory::init_heap();

    klog::init();

    log::info!("Reduct OS kernel starting...");

    acpi::init();

    smp::CPUS.write().load(*BSP_LAPIC_ID);
    irq::IDT.load();
    smp::CPUS.write().init_ap();

    acpi::apic::init();

    syscall::init();
    task::init();

    fs::init();

    module::load_all_module();

    loop {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}

pub mod acpi;
pub mod fs;
pub mod gdt;
pub mod irq;
pub mod klog;
pub mod memory;
pub mod module;
pub mod serial;
pub mod smp;
pub mod syscall;
pub mod task;

pub fn addr_of<T>(reffer: &T) -> usize {
    reffer as *const T as usize
}

pub fn ref_to_mut<T>(reffer: &T) -> &mut T {
    unsafe { &mut *(addr_of(reffer) as *const T as *mut T) }
}

pub fn ref_to_static<T>(reffer: &T) -> &'static T {
    unsafe { &*(addr_of(reffer) as *const T) }
}

#[macro_export]
macro_rules! unsafe_trait_impl {
    ($struct: ident, $trait: ident) => {
        unsafe impl $trait for $struct {}
    };
    ($struct: ident, $trait: ident, $life: tt) => {
        unsafe impl<$life> $trait for $struct<$life> {}
    };
}
