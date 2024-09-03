// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

#[cfg(feature = "irox/bits")]
use irox::bits as irox_bits;

use crate::error::Error;
use irox::irox_bits::{Bits, MutBits};
use irox::irox_log::log::info;
use irox::structs::Struct;
use irox::tools::uuid::UUID;
use std::io::Write;
use windows::Win32::System::SystemInformation::{GetSystemFirmwareTable, RSMB};

pub fn read_raw_smbios_tables() -> Vec<u8> {
    let mut buf = [0u8; 1024 * 16];
    let firmware_table_provider = RSMB;
    let firmware_table_id = 0x0000;

    let p_firmware_table_buffer = Some(buf.as_mut_slice());

    let val = unsafe {
        GetSystemFirmwareTable(
            firmware_table_provider,
            firmware_table_id,
            p_firmware_table_buffer,
        )
    } as usize;

    Vec::from(buf.get(0..val).unwrap_or_default())
}

#[derive(Default, Debug, Struct)]
#[little_endian]
pub struct SMBIOSHeader {
    pub used_calling_method: u8,
    pub smbios_major_version: u8,
    pub smbios_minor_version: u8,
    pub dmi_revision: u8,
    pub table_data_length: u32,
}

pub fn read_next_table<T: MutBits + Bits>(val: &mut T) -> Result<SMBiosTable, Error> {
    let smtype = val.read_u8()?;
    let len = val.read_u8()? - 2;

    match smtype {
        0 => Ok(SMBiosTable::BiosInformation(BIOSInformation::parse_from(
            val, len,
        )?)),
        1 => Ok(SMBiosTable::SystemInformation(
            SystemInformation::parse_from(val)?,
        )),
        2 => Ok(SMBiosTable::BaseboardInformation(
            BaseboardInformation::parse_from(val)?,
        )),

        e => Ok(SMBiosTable::Unknown(e)),
    }
}

macro_rules! get_str_from_table {
    ($table:ident,$index:ident) => {
        ($index > 0).then(|| $table.get($index as usize - 1).cloned().unwrap_or_default())
    };
}

#[derive(Debug)]
pub enum SMBiosTable {
    BiosInformation(BIOSInformation),
    SystemInformation(SystemInformation),
    BaseboardInformation(BaseboardInformation),
    Unknown(u8),
}

#[derive(Debug)]
pub struct BIOSInformation {
    pub handle: u16,
    pub vendor_str: Option<String>,
    pub bios_version_str: Option<String>,
    pub bios_addr: u16,
    pub bios_date_str: Option<String>,
    pub bios_rom_size: u8,
    pub bios_characteristics: u32,
    pub bios_major_release: u8,
    pub bios_minor_release: u8,
    pub embfirm_major_release: u8,
    pub embfirm_minor_release: u8,
    pub extbios_rom_size: u16,
}
impl BIOSInformation {
    pub fn parse_from<T: Bits + MutBits>(val: &mut T, len: u8) -> Result<BIOSInformation, Error> {
        let handle = val.read_le_u16()?;
        let vendor_stridx = val.read_u8()?;
        let bios_version_stridx = val.read_u8()?;
        let bios_addr = val.read_le_u16()?;
        let bios_date_stridx = val.read_u8()?;
        let bios_rom_size = val.read_u8()?;
        let bios_characteristics = val.read_le_u32()?;

        let ext_bits = len.saturating_sub(18) as usize;
        info!("ext bits: {ext_bits}");
        val.advance(ext_bits)?;

        let bios_major_release = val.read_u8()?;
        let bios_minor_release = val.read_u8()?;
        let embfirm_major_release = val.read_u8()?;
        let embfirm_minor_release = val.read_u8()?;
        let extbios_rom_size = val.read_le_u16()?;

        let strs = read_str_table(val)?;
        let vendor_str = get_str_from_table!(strs, vendor_stridx);
        let bios_version_str = get_str_from_table!(strs, bios_version_stridx);
        let bios_date_str = get_str_from_table!(strs, bios_date_stridx);

        Ok(BIOSInformation {
            handle,
            vendor_str,
            bios_version_str,
            bios_addr,
            bios_date_str,
            bios_rom_size,
            bios_characteristics,

            bios_major_release,
            bios_minor_release,
            embfirm_major_release,
            embfirm_minor_release,
            extbios_rom_size,
        })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum WakeUpType {
    Reserved = 0u8,
    Other = 1u8,
    Unknown = 2u8,
    APMTimer = 3u8,
    ModemRing = 4u8,
    LANRemote = 5u8,
    PowerSwitch = 6u8,
    PCIPME = 7u8,
    ACPowerRestored = 8u8,
}
impl From<u8> for WakeUpType {
    fn from(value: u8) -> Self {
        match value {
            0 => WakeUpType::Reserved,
            2 => WakeUpType::Unknown,
            3 => WakeUpType::APMTimer,
            4 => WakeUpType::ModemRing,
            5 => WakeUpType::LANRemote,
            6 => WakeUpType::PowerSwitch,
            7 => WakeUpType::PCIPME,
            8 => WakeUpType::ACPowerRestored,
            _ => WakeUpType::Other,
        }
    }
}

#[derive(Debug)]
pub struct SystemInformation {
    pub handle: u16,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub version: Option<String>,
    pub serial_number: Option<String>,
    pub uuid: UUID,
    pub wake_up_type: WakeUpType,
    pub sku_number: Option<String>,
    pub family: Option<String>,
}

impl SystemInformation {
    pub fn parse_from<T: Bits + MutBits>(val: &mut T) -> Result<SystemInformation, Error> {
        let handle = val.read_le_u16()?;
        let manuf_stridx = val.read_u8()?;
        let prodname_stridx = val.read_u8()?;
        let version_stridx = val.read_u8()?;
        let sernum_stridx = val.read_u8()?;
        let uuid_time_low = val.read_le_u32()?;
        let uuid_time_med = val.read_le_u16()?;
        let uuid_time_high = val.read_le_u16()?;
        let uuid_clk_hi = val.read_u8()?;
        let uuid_clk_low = val.read_u8()?;
        let node = val.read_exact::<6>()?;
        let mut uuid = [0u8; 16];

        let mut uuidwr = uuid.as_mut_slice();
        uuidwr.write_be_u32(uuid_time_low)?;
        uuidwr.write_be_u16(uuid_time_med)?;
        uuidwr.write_be_u16(uuid_time_high)?;
        uuidwr.write_u8(uuid_clk_hi)?;
        uuidwr.write_u8(uuid_clk_low)?;
        uuidwr.write_all(&node)?;

        let uuid = UUID::from(uuid);
        let wake_up_type: WakeUpType = val.read_u8()?.into();
        let sku_num_stridx = val.read_u8()?;
        let family_stridx = val.read_u8()?;

        let table = read_str_table(val)?;
        let manufacturer = get_str_from_table!(table, manuf_stridx);
        let product_name = get_str_from_table!(table, prodname_stridx);
        let version = get_str_from_table!(table, version_stridx);
        let serial_number = get_str_from_table!(table, sernum_stridx);
        let sku_number = get_str_from_table!(table, sku_num_stridx);
        let family = get_str_from_table!(table, family_stridx);
        Ok(SystemInformation {
            handle,
            manufacturer,
            product_name,
            version,
            serial_number,
            uuid,
            wake_up_type,
            sku_number,
            family,
        })
    }
}

#[derive(Debug)]
pub struct BaseboardInformation {
    pub handle: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub serial_number: Option<String>,
    pub asset_tag: Option<String>,
    pub feature_flags: u8,
    pub location_in_chassis: Option<String>,
    pub chassis_handle: u16,
    pub board_type: u8,
    pub object_handles: Vec<u16>,
}
impl BaseboardInformation {
    pub fn parse_from<T: Bits + MutBits>(val: &mut T) -> Result<BaseboardInformation, Error> {
        let handle = val.read_le_u16()?;
        let manuf_stridx = val.read_u8()?;
        let product_stridx = val.read_u8()?;
        let version_stridx = val.read_u8()?;
        let serno_stridx = val.read_u8()?;
        let assettag_stridx = val.read_u8()?;
        let feature_flags = val.read_u8()?;
        let location_stridx = val.read_u8()?;
        let chassis_handle = val.read_le_u16()?;
        let board_type = val.read_u8()?;

        let num_obj_handles = val.read_u8()?;
        let mut object_handles = Vec::with_capacity(num_obj_handles as usize);
        for _ in 0..num_obj_handles {
            object_handles.push(val.read_le_u16()?);
        }
        let table = read_str_table(val)?;
        let manufacturer = get_str_from_table!(table, manuf_stridx);
        let product = get_str_from_table!(table, product_stridx);
        let version = get_str_from_table!(table, version_stridx);
        let serial_number = get_str_from_table!(table, serno_stridx);
        let asset_tag = get_str_from_table!(table, assettag_stridx);
        let location_in_chassis = get_str_from_table!(table, location_stridx);

        Ok(BaseboardInformation {
            handle,
            manufacturer,
            product,
            version,
            serial_number,
            asset_tag,
            feature_flags,
            location_in_chassis,
            chassis_handle,
            board_type,
            object_handles,
        })
    }
}

fn read_null_terminated_str<T: Bits + MutBits>(val: &mut T) -> Result<String, Error> {
    let mut out = String::new();
    loop {
        let read = val.read_u8()?;
        if read == 0 {
            break;
        }
        out.push(read as char);
    }
    Ok(out)
}

fn read_str_table<T: Bits + MutBits>(val: &mut T) -> Result<Vec<String>, Error> {
    let mut strs = Vec::new();
    loop {
        let read_str = read_null_terminated_str(val)?;
        if read_str.is_empty() {
            break;
        }
        strs.push(read_str);
    }
    Ok(strs)
}
