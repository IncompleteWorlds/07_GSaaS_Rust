/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service 
 * Common functions to all modules
 */
 
use std::num::{ParseIntError};
use std::fmt::Write;
use std::result::Result;


// limit the maximum amount of data that server will accept
pub const MAX_SIZE_JSON : usize =  262_144;


#[derive(Clone)]
pub enum EnumStatus {
    NONE,
    RUNNING,
    STOPPED,
}

impl EnumStatus {
    pub fn to_string(&self) -> String {
        match *self {
            EnumStatus::NONE     => String::from("None"),
            EnumStatus::RUNNING  => String::from("Running"),
            EnumStatus::STOPPED  => String::from("Stopped"),
        }
    }
}



//
// ====================================================================
// ====================================================================
//
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
    .step_by(2)
    .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
    .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b);
    }
    s
}
