#![no_std]
#![no_main]
#![allow(unused_variables)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
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
pub mod gdt;
pub mod irq;
pub mod klog;
pub mod memory;
pub mod serial;
pub mod smp;
