use std::{error::Error, fmt::Display, iter::repeat};

use base64::{engine::general_purpose, Engine};
use constant_time_eq::constant_time_eq;
use rand::{rngs::OsRng, RngCore};
pub use scrypt::{scrypt, Params};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct InsignoScryptParams {
    pub(crate) log_n: u8,
    pub(crate) r: u32,
    pub(crate) p: u32,
    pub(crate) len: usize,
}
impl Default for InsignoScryptParams {
    fn default() -> Self {
        Self {
            log_n: 15,
            r: 8,
            p: 1,
            len: 30,
        }
    }
}
impl From<InsignoScryptParams> for Params {
    fn from(val: InsignoScryptParams) -> Params {
        Params::new(val.log_n, val.r, val.p, val.len).unwrap()
    }
}

pub fn scrypt_simple(password: &str, params: &Params) -> Result<String, Box<dyn Error>> {
    // 128-bit salt
    let mut salt: Vec<u8> = [0u8; 16].to_vec();
    OsRng.fill_bytes(&mut salt);

    // 256-bit derived key
    let mut dk = [0u8; 32];

    scrypt(password.as_bytes(), &salt, params, &mut dk)?;

    let mut result = "$rscrypt$".to_string();
    if params.r() < 256 && params.p() < 256 {
        result.push_str("0$");
        let mut tmp = [0u8; 3];
        tmp[0] = params.log_n();
        tmp[1] = params.r() as u8;
        tmp[2] = params.p() as u8;

        result.push_str(&general_purpose::STANDARD.encode(tmp));
    } else {
        result.push_str("1$");
        let mut tmp = [0u8; 9];
        tmp[0] = params.log_n();
        tmp[1..5].swap_with_slice(&mut params.r().to_le_bytes());
        tmp[5..9].swap_with_slice(&mut params.p().to_le_bytes());
        result.push_str(&general_purpose::STANDARD.encode(tmp));
    }
    result.push('$');
    result.push_str(&general_purpose::STANDARD.encode(salt));
    result.push('$');
    result.push_str(&general_purpose::STANDARD.encode(dk));
    result.push('$');

    Ok(result)
}

#[derive(Debug, Copy, Clone)]
pub struct ScryptError {}
impl Display for ScryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash is not in Rust Scrypt format.")
    }
}
impl Error for ScryptError {}
/**
 * scrypt_check compares a password against the result of a previous call to scrypt_simple and
 * returns true if the passed in password hashes to the same value.
 *
 * # Arguments
 *
 * * password - The password to process as a str
 * * hashed_value - A string representing a hashed password returned by scrypt_simple()
 *
 */
pub fn scrypt_check(password: &str, hashed_value: &str) -> Result<bool, ScryptError> {
    let err: ScryptError = ScryptError {};
    let mut iter = hashed_value.split('$');
    // Check that there are no characters before the first "$"
    match iter.next() {
        Some(x) => {
            if !x.is_empty() {
                return Err(err);
            }
        }
        None => return Err(err),
    }

    // Check the name
    match iter.next() {
        Some(t) => {
            if t != "rscrypt" {
                return Err(err);
            }
        }
        None => return Err(err),
    }

    // Parse format - currenlty only version 0 (compact) and 1 (expanded) are supported
    let params: Params;
    match iter.next() {
        Some(fstr) => {
            // Parse the parameters - the size of them depends on the if we are using the compact or
            // expanded format

            let pvec = match iter.next() {
                Some(pstr) => match general_purpose::STANDARD.decode(pstr) {
                    Ok(x) => x,
                    Err(_) => return Err(err),
                },
                None => return Err(err),
            };
            match fstr {
                "0" => {
                    if pvec.len() != 3 {
                        return Err(err);
                    }
                    let log_n = pvec[0];
                    let r = pvec[1] as u32;
                    let p = pvec[2] as u32;
                    params = Params::new(log_n, r, p, 32usize).map_err(|_| err)?;
                }
                "1" => {
                    if pvec.len() != 9 {
                        return Err(err);
                    }
                    let log_n = pvec[0];
                    let r = u32::from_le_bytes(pvec[1..5].try_into().unwrap());
                    let p = u32::from_le_bytes(pvec[5..9].try_into().unwrap());
                    params = Params::new(log_n, r, p, 32usize).map_err(|_| err)?;
                }
                _ => return Err(err),
            }
        }
        None => return Err(err),
    }

    // Salt
    let salt = match iter.next() {
        Some(sstr) => match general_purpose::STANDARD.decode(sstr) {
            Ok(salt) => salt,
            Err(_) => return Err(err),
        },
        None => return Err(err),
    };

    // Hashed value
    let hash = match iter.next() {
        Some(hstr) => match general_purpose::STANDARD.decode(hstr) {
            Ok(hash) => hash,
            Err(_) => return Err(err),
        },
        None => return Err(err),
    };

    // Make sure that the input ends with a "$"
    match iter.next() {
        Some(x) => {
            if !x.is_empty() {
                return Err(err);
            }
        }
        None => return Err(err),
    }

    // Make sure there is no trailing data after the final "$"
    if iter.next().is_some() {
        return Err(err);
    }

    let mut output: Vec<u8> = repeat(0).take(hash.len()).collect();
    scrypt(password.as_bytes(), &salt, &params, &mut output).map_err(|_| err)?;

    // Be careful here - its important that the comparison be done using a fixed time equality
    // check. Otherwise an adversary that can measure how long this step takes can learn about the
    // hashed value which would allow them to mount an offline brute force attack against the
    // hashed password.
    Ok(constant_time_eq(&output, &hash))
}

/*#[cfg(test)]
pub mod test {
    use std::time::Instant;

    use rand::{rngs::OsRng, RngCore, Rng, thread_rng, distributions::Alphanumeric};
    use super::*;
    #[test]
    pub fn sicurezza(){

        for i in 0..100000{
            println!("{i}");
            let mut rng = thread_rng();
            let len = (20) as usize;
            let passw: String = rng.sample_iter(&Alphanumeric)
                .take(len)
                .map(char::from)
                .collect();
            let par = Params::new(17, 8, 1, 32).unwrap();
            let start_encode = Instant::now();
            let hashed_passw = scrypt_simple(&passw, &par).unwrap();
            let encode_time = start_encode.elapsed().as_millis();
            let start_decode = Instant::now();
            assert!(scrypt_check(&passw, &hashed_passw).unwrap());
            println!("encode {} decode {}, {}", encode_time, start_decode.elapsed().as_millis(), hashed_passw);
        }

    }
}*/
