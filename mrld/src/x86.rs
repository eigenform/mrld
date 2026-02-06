
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

