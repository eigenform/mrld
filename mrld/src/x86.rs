
pub mod mtrr;
pub mod msr;
pub mod cr;
pub mod segment;
pub mod dtr; 
pub mod idt;
pub mod gdt;
pub mod gpr;
pub mod io;

pub mod apic;

pub use cr::*;
pub use dtr::*;
pub use msr::*;
pub use gpr::*;
pub use io::*;

use core::arch::x86_64::CpuidResult;

#[inline(always)]
pub fn mfence() {
    unsafe { core::arch::asm!("mfence") }
}

#[inline(always)]
pub fn lfence() {
    unsafe { core::arch::asm!("lfence") }
}

#[inline(always)]
pub fn invd() {
    unsafe { core::arch::asm!("invd") }
}

#[inline(always)]
pub fn wbinvd() {
    unsafe { core::arch::asm!("wbinvd") }
}

#[inline(always)]
pub fn pause() {
    unsafe { core::arch::asm!("pause") }
}

#[inline(always)]
pub unsafe fn hlt() {
    unsafe { core::arch::asm!("hlt") }
}

#[inline(always)]
pub fn cpuid(leaf: u32, subleaf: u32) -> CpuidResult { 
    core::arch::x86_64::__cpuid_count(leaf, subleaf)
}




