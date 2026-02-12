
use crate::println;
use core::panic::PanicInfo;

/// Global panic handler
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! { unsafe { 
    // Disable interrupts
    core::arch::asm!("cli");

    if let Some(loc) = info.location() { 
        println!("[!] PANIC! in '{}', line {}, col {}",
            loc.file(), loc.line(), loc.column()
        );
        println!("{}", info.message());
    } else { 
        println!("[!] PANIC!: (no location)"); 
        println!("{}", info.message());
    }
    loop {}
} }


