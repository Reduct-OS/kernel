#![no_std]
#![no_main]
#![allow(unused_variables)]

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
