// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

#![allow(clippy::print_stdout)]

use irox::tools::hex::HexDump;
use irox_safe_windows::error::Error;
#[cfg(windows)]
use irox_safe_windows::smbios::{read_next_table, read_raw_smbios_tables, SMBIOSHeader};
use irox_structs::Struct;

#[cfg(windows)]
fn main() -> Result<(), Error> {
    let mut tables = read_raw_smbios_tables();
    let header = SMBIOSHeader::parse_from(&mut tables)?;

    println!("Header: {header:#?}");
    let first = read_next_table(&mut tables)?;
    println!("first table: {first:#?}");
    let second = read_next_table(&mut tables)?;
    println!("second table: {second:#?}");
    let third = read_next_table(&mut tables)?;
    println!("second table: {third:#?}");

    tables.hexdump();

    Ok(())
}

#[cfg(not(windows))]
#[allow(clippy::print_stderr)]
pub fn main() {
    eprintln!("This example only supported on windows targets.");
}
