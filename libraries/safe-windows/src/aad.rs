// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use crate::error::Error;
use windows::Win32::NetworkManagement::NetManagement::{NetFreeAadJoinInformation, NetGetAadJoinInformation};
use windows_core::PCWSTR;
use crate::types::SaferToString;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum JoinType {
    Unknown,
    Device,
    Workspace,
    Other(u8)
}
impl From<u8> for JoinType {
    fn from(value: u8) -> Self {
        match value {
            0 => JoinType::Unknown,
            1 => JoinType::Device,
            2 => JoinType::Workspace,
            _ => JoinType::Other(value)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JoinInfo {
    pub join_type: JoinType,
    pub device_id: String,
    pub user_id: String,
    pub user_key_id: String,
    pub user_key_name: String,
    pub idp_domain: String,
    pub tenant_id: String,
    pub join_user_email: String,
    pub tenant_display_name: String,
    pub mdm_enrollment_url: String,
    pub mdm_terms_of_use_url: String,
    pub mdm_compliance_url: String,
    pub user_setting_sync_url: String,
}
pub fn aad() -> Result<Option<JoinInfo>, Error>{
    unsafe {
        let tenant_id = PCWSTR::null();
        let join_info = NetGetAadJoinInformation(tenant_id)?;

        if join_info.is_null() {
            return Ok(None);
        }
        let ji = *join_info;
        let ui = ji.pUserInfo;
        if ui.is_null() {
            return Ok(None);
        }
        let ui = *ui;

        // let jc = *ji.pJoinCertificate;
        // let ci = *jc.pCertInfo;
        // let serial = slice_from_raw_parts(ci.SerialNumber.pbData, ci.SerialNumber.cbData as usize);
        // let Some(serial) = serial.as_ref() else {
        //     return Ok(None);
        // };
        // let mut serial = Vec::from(serial);
        // serial.reverse();
        // let serial = to_hex_str_upper(&serial);
        //
        // println!("{serial} {:#?}", ci);

        let out = JoinInfo {
            join_type: (ji.joinType.0 as u8).into(),
            device_id: ji.pszDeviceId.to_string_safer(),
            user_id: ui.pszUserEmail.to_string_safer(),
            user_key_id: ui.pszUserKeyId.to_string_safer(),
            user_key_name: ui.pszUserKeyName.to_string_safer(),
            idp_domain: ji.pszIdpDomain.to_string_safer(),
            tenant_id: ji.pszTenantId.to_string_safer(),
            join_user_email: ji.pszJoinUserEmail.to_string_safer(),
            tenant_display_name: ji.pszTenantDisplayName.to_string_safer(),
            mdm_enrollment_url: ji.pszMdmEnrollmentUrl.to_string_safer(),
            mdm_terms_of_use_url: ji.pszMdmTermsOfUseUrl.to_string_safer(),

            mdm_compliance_url: ji.pszMdmComplianceUrl.to_string_safer(),
            user_setting_sync_url: ji.pszUserSettingSyncUrl.to_string_safer(),
        };

        NetFreeAadJoinInformation(Some(join_info));
        Ok(Some(out))
    }
}

#[cfg(test)]
mod tests {
    use crate::aad::aad;
    use crate::error::Error;

    #[test]
    pub fn test() -> Result<(), Error>{
        let res = aad()?;
        println!("{res:#?}");
        Ok(())
    }
}