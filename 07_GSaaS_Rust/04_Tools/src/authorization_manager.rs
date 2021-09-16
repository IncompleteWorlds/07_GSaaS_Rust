/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Authorization Manager
 * It checks whether the user is authorized to request an operation
 *
 */

// Date, Time, UTC
use chrono::{Local, DateTime, Duration, Utc};

// Log 
use log::{debug, error, info, trace, warn};

// #[macro_use]
use diesel;
use diesel::prelude::*;


// Common items, users, claims
use common::claims::*;
use common::data_structs::license::*;

use crate::db::user::*;


/**
 * Check that the API call contain a valid JWT token
 */
pub fn check_authorization(conn: &SqliteConnection, in_key: &String, in_secret_key: &String) -> Result<UserDb, bool> {
    // Check the authentication key
    let the_claims = Claims::decode_token(in_key.as_str(), in_secret_key);
    match the_claims {
        Ok(c) => {
            match is_valid_claim(conn, &c) {
                Some(u) => Ok(u),
                None => Err(false),
            }
        },
        Err(_e) => Err(false),
    }
}

 /**
 * check whether claim is valid;
 * - iss = fdsaas
 * - exp = Not expired
 * - id = Valid user id
 * - sub = valid license id
 */
fn is_valid_claim(conn: &SqliteConnection, in_claim: &Claims) -> Option<UserDb> {
    // Check issuer
    if in_claim.iss != GSAAS_ISSUER {
        error!("Auth: Invalid issuer: {}", in_claim.iss);
        return None;
    }
    
    // Check expiration date
    let now: DateTime<Utc> = Utc::now(); 

    if in_claim.exp < now.timestamp() {
        error!("Auth: Token is expired");
        return None;
    }

    // Read the user data from the DB
    let tmp_user = UserDb::by_id(conn, &in_claim.id);
    if tmp_user.is_none() {
        error!("Auth: User with id: {} not found", in_claim.id);
        return None;
    }

    // Check license
    if in_claim.sub != EnumLicenseType::DemoLicense.to_string() {
        error!("Auth: Invalid license. Only Demo is implemented: {}", in_claim.sub);
        return None;
    }

    return tmp_user;
}