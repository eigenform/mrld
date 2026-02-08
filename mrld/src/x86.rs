
pub mod mtrr;
pub mod msr;
pub mod cr;
pub mod segment;
pub mod dtr; 
pub mod idt;
pub mod gdt;
pub mod gpr;
pub mod io;

pub use cr::*;
pub use dtr::*;
pub use msr::*;
pub use gpr::*;
pub use io::*;

#[inline(always)]
pub fn mfence() {
    unsafe { core::arch::asm!("mfence", options(raw)); }
}

#[inline(always)]
pub fn lfence() {
    unsafe { core::arch::asm!("lfence", options(raw)); }
}

#[inline(always)]
pub fn invd() {
    unsafe { core::arch::asm!("invd", options(raw)); }
}

#[inline(always)]
pub fn wbinvd() {
    unsafe { core::arch::asm!("wbinvd", options(raw)); }
}

#[inline(always)]
pub fn pause() {
    unsafe { core::arch::asm!("pause", options(raw)); }
}




