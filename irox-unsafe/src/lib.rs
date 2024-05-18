// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors

#[cfg(feature = "safe-linux")]
pub use irox_safe_linux as safe_linux;
#[cfg(feature = "safe-windows")]
pub use irox_safe_windows as safe_windows;
