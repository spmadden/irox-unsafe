// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

//!
//! Windows terminal IO functions
//!

use crate::error::Error;
use irox::enums as irox_enums;
use irox::enums::EnumIterItem;
use irox::tools::ToUnsigned;
use std::fmt::Debug;
use std::marker::PhantomData;
use windows::Win32::System::Console::{
    GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
    CONSOLE_SCREEN_BUFFER_INFO, STD_ERROR_HANDLE, STD_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
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

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, EnumIterItem)]
pub enum ConsoleInputModes {
    /// When set, typed stdin characters are duplicated to stdout
    EnableEchoInput = 0x0004,
    /// When set, inserted text does not overwrite the line
    EnableInsertMode = 0x0020,
    /// When set, read line-by-line.  Disabled = raw mode
    EnableLineInput = 0x0002,
    /// When set, generates mouse events
    EnableMouseInput = 0x0010,
    /// When enabled, Ctrl+C, `\r`, and `\n` are handled by the terminal
    EnableProcessedInput = 0x0001,
    /// When enabled with `EnableExtendedFlags` the mouse can select and edit text
    EnableQuickEditMode = 0x0040,
    /// When set, window resizes are reported to the app
    EnableWindowInput = 0x0008,
    /// When set, enables control character sequences
    EnableVirtualTerminalInput = 0x0200,
}
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, EnumIterItem)]
pub enum ConsoleOutputModes {
    /// When set, enable ascii control character sequences
    EnableProcessedOutput = 0x0001,
    /// When set, lines wrap at the window width
    EnableWrapAtEOLOutput = 0x0002,
    /// When set, enable VT control character sequences
    EnableVirtualTerminalProcessing = 0x0004,
    DisableNewlineAutoReturn = 0x0008,
    EnableLVBGridWorldwide = 0x0010,
}
pub trait ModeConversion: EnumIterItem {
    fn from_console(cn: CONSOLE_MODE) -> Box<[Self]>
    where
        Self: Sized;
    fn into_console(i: Box<[Self]>) -> CONSOLE_MODE
    where
        Self: Sized;
}
macro_rules! modeconv {
    ($t:ty) => {
        impl ModeConversion for $t {
            fn from_console(cn: CONSOLE_MODE) -> Box<[Self]> {
                let mut v = Vec::new();
                for el in Self::iter_items() {
                    let e = el as u32;
                    if e & cn.0 == e {
                        v.push(el);
                    }
                }
                v.into_boxed_slice()
            }
            fn into_console(i: Box<[Self]>) -> CONSOLE_MODE {
                let mut out = CONSOLE_MODE(0);
                for el in i {
                    out.0 |= el as u32;
                }
                out
            }
        }
        impl ToUnsigned for $t {
            type Output = u32;
            fn to_unsigned(self) -> u32 {
                self as u32
            }
        }
    };
}
modeconv!(ConsoleOutputModes);
modeconv!(ConsoleInputModes);

pub struct ConsoleStream<T: ModeConversion + Debug> {
    handle: STD_HANDLE,
    _phantom: PhantomData<T>,
}
impl<T: ModeConversion + Debug + ToUnsigned<Output = u32> + Eq> ConsoleStream<T> {
    pub fn get_mode(&self) -> Result<Box<[T]>, Error> {
        let mut v = CONSOLE_MODE(0);
        unsafe {
            let Ok(hnd) = GetStdHandle(self.handle) else {
                return Error::no_console();
            };
            GetConsoleMode(hnd, &mut v)?;
        }

        Ok(T::from_console(v))
    }
    pub fn set_mode(&self, mode: Box<[T]>) -> Result<(), Error> {
        unsafe {
            let Ok(hnd) = GetStdHandle(self.handle) else {
                return Error::no_console();
            };
            let mode = T::into_console(mode);
            SetConsoleMode(hnd, mode)?;
        }
        Ok(())
    }
    pub fn enable_mode(&self, mode: T) -> Result<(), Error> {
        let mut v = CONSOLE_MODE(0);
        unsafe {
            let Ok(hnd) = GetStdHandle(self.handle) else {
                return Error::no_console();
            };
            GetConsoleMode(hnd, &mut v)?;
            v.0 |= mode.to_unsigned();
            SetConsoleMode(hnd, v)?;
        }
        Ok(())
    }
    pub fn disable_mode(&self, mode: &T) -> Result<(), Error> {
        let mut v = CONSOLE_MODE(0);
        let modes = self.get_mode()?;
        for m in modes {
            if *mode != m {
                v.0 |= m.to_unsigned();
            }
        }
        unsafe {
            let Ok(hnd) = GetStdHandle(self.handle) else {
                return Error::no_console();
            };
            SetConsoleMode(hnd, v)?;
        }
        Ok(())
    }
    ///
    /// Queries the current console window for it's information.
    pub fn get_console_info(&self) -> Result<WindowsConsoleInfo, Error> {
        let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
        unsafe {
            let Ok(hnd) = GetStdHandle(self.handle) else {
                return Error::no_console();
            };
            GetConsoleScreenBufferInfo(hnd, &mut info)?;
        }
        Ok(info.into())
    }

    #[allow(clippy::print_stderr)]
    #[allow(clippy::print_stdout)]
    pub fn dump_info(&self) -> Result<(), Error> {
        match self.get_console_info() {
            Ok(i) => {
                println!("{i:#?}");
            }
            Err(e) => {
                eprintln!("Could not get console info: {e}");
            }
        }
        match self.get_mode() {
            Ok(i) => {
                println!("{i:#?}");
            }
            Err(e) => {
                eprintln!("Could not get console mode: {e}");
            }
        }
        Ok(())
    }
}
#[allow(clippy::print_stderr)]
#[allow(clippy::print_stdout)]
pub fn dump_console_info() -> Result<(), Error> {
    if let Err(e) = ConsoleStream::get_stdin().dump_info() {
        eprintln!("{e}");
    }
    if let Err(e) = ConsoleStream::get_stdout().dump_info() {
        eprintln!("{e}");
    }
    if let Err(e) = ConsoleStream::get_stderr().dump_info() {
        eprintln!("{e}");
    }
    Ok(())
}
impl ConsoleStream<ConsoleOutputModes> {
    pub fn get_stdout() -> Self {
        ConsoleStream {
            handle: STD_OUTPUT_HANDLE,
            _phantom: PhantomData,
        }
    }
    pub fn get_stderr() -> ConsoleStream<ConsoleOutputModes> {
        ConsoleStream {
            handle: STD_ERROR_HANDLE,
            _phantom: PhantomData,
        }
    }
}
impl ConsoleStream<ConsoleInputModes> {
    pub fn get_stdin() -> ConsoleStream<ConsoleInputModes> {
        ConsoleStream {
            handle: STD_INPUT_HANDLE,
            _phantom: PhantomData,
        }
    }
}
