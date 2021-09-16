/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * 
 */
use std::result::Result;

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json;

//use std::collections::HashMap;



// =======================================================
// Users
// =======================================================

/**
 * This is a special message. Message for registering a new user
 * It will return the new user id and his authentication key
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterStruct {
    pub username:   String,
    // Password shall be already hashed with SHA-256
    pub password:   String,
    pub email:      String,
}
 
/**
 * Response to Register message
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseStruct {
    pub user_id:    String,
}
 
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginStruct {
    pub username_email:   String,
    pub password:         String,
}
 
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseStruct {
    pub user_id:          String,
    pub jwt_token:        String,
}
 
// Logout
#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutStruct {
    pub user_id:          String,
}
 
// Logout response
// None

// De-register
#[derive(Serialize, Deserialize, Debug)]
pub struct DeregisterStruct {
    pub user_id:          String,
}
 
// De-register response
// None
 
 
