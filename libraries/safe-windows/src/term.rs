// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

//!
//! Windows terminal IO functions
//!

use crate::error::Error;
use windows::Win32::System::Console::{
    GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_OUTPUT_HANDLE,
};

/// Information returned from [`get_console_info`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WindowsConsoleInfo {
    pub window_size_x: i16,
    pub window_size_y: i16,
    pub window_max_size_x: i16,
    pub window_max_size_y: i16,
    pub cursor_pos_x: i16,
    pub cursor_pos_y: i16,
}
impl From<CONSOLE_SCREEN_BUFFER_INFO> for WindowsConsoleInfo {
    fn from(value: CONSOLE_SCREEN_BUFFER_INFO) -> Self {
        Self {
            window_size_x: value.dwSize.X,
            window_size_y: value.dwSize.Y,
            cursor_pos_x: value.dwCursorPosition.X,
            cursor_pos_y: value.dwCursorPosition.Y,
            window_max_size_x: value.dwMaximumWindowSize.X,
            window_max_size_y: value.dwMaximumWindowSize.Y,
        }
    }
}

///
/// Queries the current console window for it's information.
pub fn get_console_info() -> Result<WindowsConsoleInfo, Error> {
    let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
    unsafe {
        let Ok(hnd) = GetStdHandle(STD_OUTPUT_HANDLE) else {
            return Error::no_console();
        };
        GetConsoleScreenBufferInfo(hnd, &mut info)?;
    }
    Ok(info.into())
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::term::get_console_info;

    #[test]
    pub fn test() -> Result<(), Error> {
        println!("{:#?}", get_console_info()?);
        Ok(())
    }
}
