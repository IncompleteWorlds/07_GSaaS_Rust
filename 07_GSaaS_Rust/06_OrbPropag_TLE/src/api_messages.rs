/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Data structures used in the TM Decoder
 */
// use std::result::Result;

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
// use serde_json;

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
    pub license:          String,
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
 
 
// =======================================================
// Missions
// =======================================================
 
// Create Mission

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMissionStruct {
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub launch_date:     String, 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMissionReponseStruct {
    pub mission_id:      String,
}

// Get Mission(s)
#[derive(Serialize, Deserialize, Debug)]
pub struct GetMissionStruct {
    pub mission_id:      String,
}

// Delete Mission
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteMissionStruct {
    pub mission_id:      String,
}

// Response
// None

// 
 
// =======================================================
// Satellites
// =======================================================
 
// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSatelliteStruct {
    pub mission_id:      String,
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub launch_date:     String, 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSatelliteReponseStruct {
    pub satellite_id:    String,
}


// Read
#[derive(Serialize, Deserialize, Debug)]
pub struct GetSatelliteStruct {
    pub satellite_id:      String,
}

// Update


// Delete
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteSatelliteStruct {
    pub satellite_id:      String,
}

// Find by id
// Find by name
 
// =======================================================
// Ground Stations
// =======================================================
 
// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroundStationStruct {    
    pub name:           String,
    pub owner:          String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroundStationResponseStruct {    
    pub ground_station_id:     String,
}

// Read
// Update
// Delete
 
// Find by id
// Find by name
 

// =======================================================
// Antennas
// =======================================================


#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAntennaStruct {    
    pub name:           String,
    pub station_id:     String,
    // WGS84 degrees
    pub latitude:       f64,
    // WGS84 degrees
    pub longitude:      f64,
    // Meters
    pub altitude:       f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAntennaResponseStruct {    
    pub antenna_id:     String,
}

// Add Elevation Mask to the stations
// Read Elevation Mask to the stations
// Delete Elevation Mask to the stations


