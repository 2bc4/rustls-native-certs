use crate::load_pem_certs;

use std::io::Error;

pub fn load_native_certs() -> Result<Vec<u8>, Error> {
    let likely_locations = openssl_probe::probe();

    match likely_locations.cert_file {
        Some(cert_file) => load_pem_certs(&cert_file),
        None => Ok(Vec::new()),
    }
}
