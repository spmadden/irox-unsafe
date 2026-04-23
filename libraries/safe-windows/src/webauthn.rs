// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use std::slice::from_raw_parts;
use windows::core::PCWSTR;
use windows::Win32::Networking::WindowsWebServices::{WebAuthNFreePlatformCredentialList, WebAuthNGetApiVersionNumber, WebAuthNGetPlatformCredentialList, WEBAUTHN_CREDENTIAL_DETAILS, WEBAUTHN_GET_CREDENTIALS_OPTIONS, WEBAUTHN_RP_ENTITY_INFORMATION, WEBAUTHN_USER_ENTITY_INFORMATION};
use crate::error::Error;

trait SaferToString {
    fn safer_to_string(&self) -> Option<String>;
}
impl SaferToString for PCWSTR {
    fn safer_to_string(&self) -> Option<String> {
        if self.is_null() {
            return None;
        }
        unsafe {
            Some(self.to_string().unwrap_or_default())
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityInformation {
    pub version: u32,
    pub id: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
}
impl From<*mut WEBAUTHN_RP_ENTITY_INFORMATION> for EntityInformation {
    fn from(value: *mut WEBAUTHN_RP_ENTITY_INFORMATION) -> Self {
        let value = unsafe { &*value };
        let id = value.pwszId.safer_to_string();
        let name = value.pwszName.safer_to_string();
        let icon = value.pwszIcon.safer_to_string();
        EntityInformation {
            version: value.dwVersion,
            id,
            name,
            icon,
        }
    }
}
#[derive(Debug, Clone)]
pub struct UserInformation {
    pub version: u32,
    pub id: u32,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub icon: Option<String>,
}
impl From< *mut WEBAUTHN_USER_ENTITY_INFORMATION> for UserInformation {
    fn from(value: *mut WEBAUTHN_USER_ENTITY_INFORMATION) -> Self {
        let value = unsafe { &*value };
        let name = value.pwszName.safer_to_string();
        let display_name = value.pwszDisplayName.safer_to_string();
        let icon = value.pwszIcon.safer_to_string();
        UserInformation {
            version: value.dwVersion,
            id: value.cbId,
            name,
            display_name,
            icon,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CredentialDetails {
    pub version: u32,
    pub credential_id: u32,
    pub entity_information: EntityInformation,
    pub user_information: UserInformation,
    pub removable: bool,
    pub backed_up: bool,
}

impl From<*mut WEBAUTHN_CREDENTIAL_DETAILS> for CredentialDetails {
    fn from(value: *mut WEBAUTHN_CREDENTIAL_DETAILS) -> Self {
        unsafe {
            let value = *value;
            
            CredentialDetails {
                version: value.dwVersion,
                credential_id: value.cbCredentialID,
                entity_information: value.pRpInformation.into(),
                user_information: value.pUserInformation.into(),
                removable: value.bRemovable.as_bool(),
                backed_up: value.bBackedUp.as_bool(),
            }
        }
    }
}

pub fn get_version_number() -> u32 {
    unsafe {
        WebAuthNGetApiVersionNumber()
    }
}
pub fn list_credentials() -> Result<(), Error>{
    // let v = "test".encode_utf16().collect::<Vec<_>>();
    // let v= v.as_ptr();
    let opts = WEBAUTHN_GET_CREDENTIALS_OPTIONS {
        dwVersion: 7,
        // pwszRpId: PCWSTR(v),
      ..Default::default()
    };
    let res = unsafe {
        WebAuthNGetPlatformCredentialList(&opts)?
    };

    unsafe {
        let deets = (*res).ppCredentialDetails;
        let vals = from_raw_parts(deets, (*res).cCredentialDetails as usize);
        for val in vals {
            let val = *val;
            let v : CredentialDetails = val.into();
            println!("{v:#?}");
        }

    }

    unsafe {
        WebAuthNFreePlatformCredentialList(res);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::webauthn::{get_version_number, list_credentials};

    #[test]
    pub fn test() -> Result<(), Error>{
        println!("{:#?}", get_version_number());
        list_credentials()?;
        Ok(())
    }
}