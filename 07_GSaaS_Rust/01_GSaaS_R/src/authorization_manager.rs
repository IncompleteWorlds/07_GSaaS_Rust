/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Authorization Manager
 * It checks whether the user is authorized to send an operation
 */

// Date, Time, UTC
use chrono::{Local, DateTime, Duration, Utc};

// Log 
use log::{debug, error, info, trace, warn};

#[macro_use]
use diesel;
use diesel::prelude::*;

use db::users::*;
use claims::*;


/**
 * Check that the API call contain a valid JWT token
 */
pub fn check_authorization(conn: &SqliteConnection, in_key: &String) -> Result<User, bool> {
    // Check the authentication key
    let the_claims = Claims::decode_token(in_key.as_str());
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
fn is_valid_claim(conn: &SqliteConnection, in_claim: &Claims) -> Option<User> {
    // Check issuer
    if in_claim.iss != FDSAAS_ISSUER {
        error!("Invalid issuer: {}", in_claim.iss);
        return None;
    }
    
    // Check expiration date
    let now: DateTime<Utc> = Utc::now(); 

    if in_claim.exp < now.timestamp() {
        error!("Token is expired");
        return None;
    }

    // Read the user data from the DB
    let tmp_user = User::by_id(conn, &in_claim.id);
    if tmp_user.is_none() {
        error!("User with id: {} not found", in_claim.id);
        return None;
    }

    // Check license
    if in_claim.sub != EnumLicenseType::DemoLicense.to_string() {
        error!("Invalid license. Only Demo is implemented: {}", in_claim.sub);
        return None;
    }

    return tmp_user;
}