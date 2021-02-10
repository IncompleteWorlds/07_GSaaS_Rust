/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Authorization Manager
 * It checks whether the user is authorized to send an operation
 */

use std::result::Result;

// Log 
use log::{debug, error, info, trace, warn};


pub fn check_authorization(in_key: &String) -> bool {
    if in_key.is_empty() == true || in_key != "00998844" {
        debug!("Invalid key: {}", in_key.as_str());
        false 

    } else {
        true

    }
}

