
pub mod mtrr;
pub mod msr;
pub mod cr;
pub mod segment;

pub use cr::CR3;
pub use msr::*;
pub use segment::{GDT, IDT};

