// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

#![allow(clippy::print_stdout)]

use irox_safe_windows::error::Error;
use irox_structs::Struct;
use irox_tools::hex::HexDump;

use irox_safe_windows::smbios::{read_next_table, read_raw_smbios_tables, SMBIOSHeader};

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
