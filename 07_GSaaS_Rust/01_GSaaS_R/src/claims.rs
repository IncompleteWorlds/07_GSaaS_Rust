/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * 
 * Functions to manage Users; CRUD
 */

// JSON serialization
use serde::{Deserialize, Serialize};

// SHA 256
//use sha2::{Sha256, Digest};
use chrono::{Local, DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

// Common functions
use crate::common::*;
use CONFIG_VARIABLES;

use crate::users::*;

pub const GSAAS_ISSUER : &str = "gsaas"; 

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // issuer
    pub iss:  String,
    // subject
    pub sub:  String,
    // expiry
    pub exp:  i64,
    // user id
    pub id:   String,
}

// struct to get converted to token and back
impl Claims {
    fn new(in_user: &User, in_expiration_mins: i64) -> Self {
        Claims {
            iss:     String::from(FDSAAS_ISSUER),
            sub:     in_user.license_id.clone(),
            // exp:  (Utc::now() + Duration::minutes(in_expiration_mins)).timestamp(),
            exp:     (Local::now() + Duration::minutes(in_expiration_mins)).timestamp(),
            id:      in_user.id.clone(),
        }
    }

    /**
     * Generate a JWT token from a User
     */
    pub fn create_token(in_user: &User, in_expiration_mins: i64) -> Result<String, String> {
        let claims = Claims::new(in_user, in_expiration_mins);

        encode(&Header::default(), &claims, &EncodingKey::from_secret( Claims::get_secret().as_bytes() ) )
            .map_err(|_err| String::from("ERROR: While generating Token") )
    }

    /**
     * Decode the JWT token and return the Claim
     */
    pub fn decode_token(in_token: &str) -> Result<Claims, String> {
        let _decoded = decode::<Claims>(
            in_token,
            &DecodingKey::from_secret( Claims::get_secret().as_bytes() ),
            &Validation::new(Algorithm::HS256),
        );
        match _decoded {
            Ok(decoded) => {
                // match User::by_email(
                    // self.find_user_with_email((decoded.claims.sub.to_string()).parse().unwrap()) {
                //     Ok(user) => Ok(user),
                //     Err(_) => Err( String::from("ERROR: Unable to decode Token") ),
                // }
                Ok(decoded.claims)
            },
            Err(_) => Err( String::from("ERROR: Invalid Token") ),
        }
    }

    /**
     * Read the configuration variable Secret_key
     */
    fn get_secret() -> String {
        let tmp_config_data = CONFIG_VARIABLES.read().unwrap();
    
        let key = tmp_config_data.secret_key.clone(); 
    
        key
    }
}
