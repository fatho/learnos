macro_rules! debug {
    ($($arg:tt)*) => {
        {
            let mut writer = crate::vga::writer();
            let mut com1 = crate::serial::COM1.lock();
            core::fmt::Write::write_fmt(&mut writer, format_args_nl!($($arg)*)).unwrap_or(());
            core::fmt::Write::write_fmt(&mut *com1, format_args_nl!($($arg)*)).unwrap_or(());
        }
    };
}