#![no_std]
#![no_main]

mod serial;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {  
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { 
    loop {}
}
