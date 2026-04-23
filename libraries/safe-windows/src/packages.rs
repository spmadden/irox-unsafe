// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use windows::core::HSTRING;
use windows::Management::Deployment::PackageManager;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Package {
    pub version: String,
    pub name: String,
    pub resource_id: String,
    pub publisher_id: String,
    pub full_name: String,
    pub family_name: String,
    pub product_id: String,
    pub display_name: String,
    pub description: String,
    pub publisher: String,
    pub author: String,
}
pub trait UnwrapString {
     fn unwrap_string(&self) -> String;
}
impl UnwrapString for windows::core::Result<HSTRING> {
    fn unwrap_string(&self) -> String {
        let Ok(v) = self else {
            return Default::default()
        };
        v.to_string()
    }
}
impl TryFrom<windows::ApplicationModel::Package> for Package {
    type Error = crate::error::Error;

    fn try_from(value: windows::ApplicationModel::Package) -> Result<Self, Self::Error> {
        let id = value.Id()?;
        let version = id.Version()?;
        let version = format!("{}.{}.{}-{}",
                              version.Major,
                              version.Minor,
                              version.Revision,
                              version.Build);
        let display_name = value.DisplayName().unwrap_string();
        let description = value.Description().unwrap_string();
        let name = id.Name().unwrap_string();
        let resource_id = id.ResourceId().unwrap_string();
        let publisher = id.Publisher().unwrap_string();
        let author = id.Author().unwrap_string();
        let publisher_id = id.PublisherId().unwrap_string();
        let full_name = id.FullName().unwrap_string();
        let family_name = id.FamilyName().unwrap_string();
        let product_id = id.ProductId().unwrap_string();
        Ok(Self {
            name,

            resource_id,
            publisher_id,
            full_name,
            family_name,
            version,
            display_name,
            description,
            product_id,
            publisher,
            author
        })
    }
}
pub fn list_packages() -> Result<(), Error> {
    let mgr = PackageManager::new()?;
    let _s = HSTRING::default();
    for pkg in mgr.FindPackages()? {
        let pkg: Package = pkg.try_into()?;
        let lower = pkg.full_name.to_lowercase();
        if lower.contains("microsoft") || lower.contains("windows") {
            continue;
        }

        if pkg.name.to_lowercase().contains("vim") || pkg.full_name.to_lowercase().contains("vim"){
        }
        println!("{pkg:#?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::packages::list_packages;

    #[test]
    #[ignore]
    pub fn test() -> Result<(), Error> {
        list_packages()?;
        Ok(())
    }
}

