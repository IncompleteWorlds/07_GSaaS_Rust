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
use common::http_errors::{self, HttpServiceError};

use crate::db::user::*;


/**
 * Check that the API call contain a valid JWT token
 */
pub fn check_authorization(conn: &SqliteConnection, in_key: &String, in_secret_key: &String,
    in_msg_id: String) -> Result<UserDb, HttpServiceError> 
{
    // Check the authentication key
    let the_claims = Claims::decode_token(in_key.as_str(), in_secret_key);
    match the_claims {
        Ok(c) => {
            match is_valid_claim(conn, &c, in_msg_id) {
                Ok(u) => Ok(u),
                Err(e) => Err(e),
            }
        },
        Err(e) => {
            Err( HttpServiceError::Unauthorized(in_msg_id))
        },
    }
}

 /**
 * check whether claim is valid;
 * - iss = fdsaas
 * - exp = Not expired
 * - id = Valid user id
 * - sub = valid license id
 */
fn is_valid_claim(conn: &SqliteConnection, in_claim: &Claims, in_msg_id: String) 
     -> Result<UserDb, HttpServiceError> 
{
    // Check issuer
    if in_claim.iss != GSAAS_ISSUER {
        let error_msg = format!("Auth: Invalid issuer: {}", in_claim.iss);

        error!("{}", error_msg);

        return Err( HttpServiceError::Forbidden(in_msg_id, error_msg) );
    }
    
    // Check expiration date
    let now: DateTime<Utc> = Utc::now(); 

    if in_claim.exp < now.timestamp() {
        let error_msg = String::from("Auth: Token is expired");

        error!("{}", error_msg);

        return Err( HttpServiceError::Forbidden(in_msg_id, error_msg) );
    }

    // Read the user data from the DB
    let tmp_user = UserDb::by_id(conn, &in_claim.id);
    if tmp_user.is_none() {
        let error_msg = format!("Auth: User with id: {} not found", in_claim.id);

        error!("{}", error_msg);

        return Err( HttpServiceError::Forbidden(in_msg_id, error_msg) );
    }

    // Check license
    if in_claim.sub != EnumLicenseType::DemoLicense.to_string() {
        let error_msg = format!("Auth: Invalid license. Only Demo is implemented: {}", in_claim.sub);

        error!("{}", error_msg);

        return Err( HttpServiceError::Forbidden(in_msg_id, error_msg) );
    }

    return Ok( tmp_user.unwrap() );
}