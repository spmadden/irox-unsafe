// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use crate::system::FTimeConversions;
use irox::irox_time::epoch::WindowsNTTimestamp;
use windows::Foundation::DateTime;

impl FTimeConversions for DateTime {
    fn to_nt_timestamp(&self) -> WindowsNTTimestamp {
        let hns = self.UniversalTime;
        let sec: f64 = hns as f64 / 1e7;
        WindowsNTTimestamp::from_seconds_f64(sec)
    }
}

pub struct Certificate {

}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::system::FTimeConversions;
    use irox::time::datetime::UTCDateTime;
    use irox::time::epoch::{FromTimestamp, UnixTimestamp};
    use windows::core::HSTRING;
    use windows::Security::Cryptography::Certificates::{CertificateQuery, CertificateStores};

    #[test]
    pub fn test() -> Result<(), Error> {
        let query = CertificateQuery::new()?;

        let my = HSTRING::from("My");
        query.SetIncludeExpiredCertificates(true)?;
        query.SetIncludeDuplicates(false)?;
        query.SetStoreName(&my)?;

        let res = CertificateStores::FindAllWithQueryAsync(&query)?.get()?;
        for cert in res {
            let from = cert.ValidFrom()?.to_nt_timestamp();
            let to = cert.ValidTo()?.to_nt_timestamp();
            let from: UTCDateTime = UnixTimestamp::from_timestamp(&from).into();
            let to: UTCDateTime = UnixTimestamp::from_timestamp(&to).into();

            println!("Certificate : {} {{", cert.FriendlyName()?);
            println!("  Subject: {}", cert.Subject()?);
            println!("  Issuer: {}", cert.Issuer()?);
            println!("  Valid From: {}", from);
            println!("  Valid To: {}", to);
            println!("  Security Device: {}", cert.IsSecurityDeviceBound()?);
            println!("  Private Key: {}", cert.HasPrivateKey()?);
            if let Ok(usage) = cert.KeyUsages() {
                println!("  Key Usages:");
                println!("    Encipher Only: {}", usage.EncipherOnly()?);
                println!("    Key Encipherment: {}", usage.KeyEncipherment()?);
                println!("    Digital Signature: {}", usage.DigitalSignature()?);
                println!("    Data Encipherment: {}", usage.DataEncipherment()?);
                println!("    Key Agreement: {}", usage.KeyAgreement()?);
                println!("    Non Repudiation: {}", usage.NonRepudiation()?);
                println!("    Key Certificate Sign: {}", usage.KeyCertificateSign()?);
            }
            println!("\n");
        }

        Ok(())
    }
}
