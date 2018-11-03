macro_rules! debug {
    ($($arg:tt)*) => {
        let mut writer = crate::vga::writer();
        core::fmt::Write::write_fmt(&mut writer, format_args!($($arg)*)).unwrap_or(());
    };
}