use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use security_framework::trust_settings::{Domain, TrustSettings, TrustSettingsForCertificate};

use std::collections::HashMap;
use std::io::{Error, ErrorKind};

pub fn load_native_certs() -> Result<Vec<u8>, Error> {
    // The various domains are designed to interact like this:
    //
    // "Per-user Trust Settings override locally administered
    //  Trust Settings, which in turn override the System Trust
    //  Settings."
    //
    // So we collect the certificates in this order; as a map of
    // their DER encoding to what we'll do with them.  We don't
    // overwrite existing elements, which mean User settings
    // trump Admin trump System, as desired.

    let mut all_certs = HashMap::new();

    for domain in &[Domain::User, Domain::Admin, Domain::System] {
        let ts = TrustSettings::new(*domain);
        let iter = ts
            .iter()
            .map_err(|err| Error::new(ErrorKind::Other, err))?;

        for cert in iter {
            let der = cert.to_der();

            //convert to PEM (for curl)
            let mut pem = Vec::new();
            pem.extend_from_slice("-----BEGIN CERTIFICATE-----\n".as_bytes());
            pem.extend_from_slice(BASE64_STANDARD.encode(der).as_bytes());
            pem.extend_from_slice("\n-----END CERTIFICATE-----\n".as_bytes());

            // If there are no specific trust settings, the default
            // is to trust the certificate as a root cert.  Weird API but OK.
            // The docs say:
            //
            // "Note that an empty Trust Settings array means "always trust this cert,
            //  with a resulting kSecTrustSettingsResult of kSecTrustSettingsResultTrustRoot".
            let trusted = ts
                .tls_trust_settings_for_certificate(&cert)
                .map_err(|err| Error::new(ErrorKind::Other, err))?
                .unwrap_or(TrustSettingsForCertificate::TrustRoot);

            all_certs.entry(pem).or_insert(trusted);
        }
    }

    let mut certs = Vec::new();

    // Now we have all the certificates and an idea of whether
    // to use them.
    for (pem, trusted) in all_certs.drain() {
        use TrustSettingsForCertificate::*;
        if let TrustRoot | TrustAsRoot = trusted {
            certs.extend_from_slice(pem.as_slice());
        }
    }

    Ok(certs)
}
