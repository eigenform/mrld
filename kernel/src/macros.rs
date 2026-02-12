
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments<'_>) { 
    use core::fmt::Write;
    crate::serial::COM2.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::macros::_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ( $crate::print!("\r\n") );
    ($fmt:expr) => ( $crate::print!(concat!($fmt, "\r\n")) );
    ($fmt:expr, $($arg:tt)*) => ( 
        $crate::print!(concat!($fmt, "\r\n"), $($arg)*)
    );
}

        //$crate::macros::_print(
        //    core::format_args!("{}\n", core::format_args!($($arg)*))
        //)

