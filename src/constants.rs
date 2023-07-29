pub const INDENT_STR: &str = "    ";
pub const INDENT: &[u8] = INDENT_STR.as_bytes();
pub const INDENT_SIZE: usize = INDENT_STR.len();

#[cfg(windows)]
pub const LINE_BREAK: &[u8; 1] = b"\n";

#[cfg(windows)]
pub const LINE_BREAK_STR: &str = "\n";

#[cfg(not(windows))]
pub const LINE_BREAK: &[u8; 1] = b"\n";

#[cfg(not(windows))]
pub const LINE_BREAK_STR: &str = "\n";
