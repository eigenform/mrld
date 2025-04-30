
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments<'_>) { 
    use core::fmt::Write;
    crate::serial::COM1.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::macros::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (
        $crate::macros::_print(
            core::format_args!("{}{}", core::format_args!($($arg)*), "\n")
        )
    );
}
