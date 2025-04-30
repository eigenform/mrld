//! 'mrld' kernel. 

#![allow(unsafe_op_in_unsafe_fn)]

#![no_std]
#![no_main]

mod macros;
mod serial;
mod x86;
mod mm; 

use mrld::{
    MrldBootArgs
};


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {  
    if let Some(loc) = info.location() { 
        println!("panic! in '{}', line {}, col {}: {}",
            loc.file(), loc.line(), loc.column(), info.message()
        );
    } else { 
        println!("panic!: {}", info.message());
    }

    loop {}
}


#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(args: *const MrldBootArgs) -> ! { 
    let args = unsafe { args.as_ref().unwrap() };

    // Initialize the COM1 port
    unsafe { 
        serial::COM1.lock().init();
    }

    println!("[*] HELO from the mrld kernel :^)");

    for entry in &args.memory_map.entries {
        if !entry.is_valid() { 
            continue;
        }
        println!("{:x?}", entry);
    }

    panic!("uhhhhh");

    loop {}
}
