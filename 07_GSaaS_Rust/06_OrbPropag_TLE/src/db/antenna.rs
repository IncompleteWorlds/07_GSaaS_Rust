/**
* (c) Incomplete Worlds 2021
* Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Antennas; CRUD
 */
use core::f64;

// JSON serialization
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Log 
use log::{debug, error, info, trace, warn};

// Time
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono::format::ParseError;

// Diesel
//#[macro_use]
use diesel;
use diesel::prelude::*;

// UUID
use uuid::Uuid;

// Database
use crate::db::schema::*;

// Common functions
use common::common::*;
use common::common_messages::*;

use crate::api_messages::*;


#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_antenna"]
pub struct AntennaDb
{
    pub id:             String,
    pub name:           String,
    pub station_id:     String,
    // WGS84 degrees
    pub latitude:       f64,
    // WGS84 degrees
    pub longitude:      f64,
    // Meters
    pub altitude:       f64,
    // Format:  YYYY-MM-DDTHH:MM:SS
    //pub created:        String,
    pub created:        NaiveDateTime,
}


impl AntennaDb 
{
    /**
     * Create an empty new antenna
     */
    pub fn new() -> Self 
    {
        AntennaDb {
            id:             String::new(),
            name:           String::new(),
            station_id:     String::new(),
            latitude:       0.0,
            longitude:      0.0,
            altitude:       0.0,
            //created:        Utc::now().to_rfc3339(),
            created:        Utc::now().naive_utc(),
        }
    }

    /**
     * Create an new antenna with the basic information
     */
    fn new_data(in_name: &String, in_station_id: &String,
        in_latitude: f64, in_longitude: f64, in_altitude: f64) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        AntennaDb {
            id:             new_uuid,
            name:           in_name.clone(),
            station_id:     in_station_id.clone(),
            latitude:       in_latitude,
            longitude:      in_longitude,
            altitude:       in_altitude,
            //created:        Utc::now().to_rfc3339(),
            created:        Utc::now().naive_utc(),
        }
    }
    
    //========================================================================
    // MESSAGES
    //========================================================================

    /**
     * Create a new antenna.
     * The JSON message shall include the antenna name, antenna description and launch date
     * Document: doc/create_antenna.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_antenna",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "name" :                  "Antenna1",
            "station_id" :            "MasPalomas"
            "latitude" :              3.456,
            "longitude" :             7.896,
            "altitude" :              234.567,
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "create_antenna_response",
            "antenna_id" :            "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "error" :                 null
        }
     */
    pub fn create(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {       
        info!("Create a new antenna: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let create_message : CreateAntennaStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON CreateAntennaStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // Check parameters
        if create_message.name.is_empty() == true {
            let tmp_msg = format!("ERROR: Name is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }
        
        if create_message.station_id.is_empty() == true {
            let tmp_msg = format!("ERROR: Station Id is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        if create_message.latitude > 90.0 {
            let tmp_msg = format!("ERROR: Latitude is incorrect");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        if create_message.longitude > 180.0 {
            let tmp_msg = format!("ERROR: Longitude is incorrect");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // Check if the Antenna already exists
        let antenna_exist = AntennaDb::by_id(conn, &create_message.name);
        match antenna_exist {
            Some(_) => {
                let tmp_msg = format!("This antenna name is already in use by another antenna, please enter another name.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
            None => {
            }
        };
    

        // Create and insert the new antenna into the database
        match AntennaDb::insert_db(conn, &create_message.name, &create_message.station_id,
            create_message.latitude, create_message.longitude, create_message.altitude) {
            Ok(nm) => {
                let info_msg : String = format!("Created a new antenna with uuid: {}", nm.id);
                info!("{}", info_msg);
                
                let tmp_antenna = CreateAntennaResponseStruct { 
                    antenna_id : nm.id,
                };

                let output = RestResponse::new_value(String::from("create_antenna_response"), in_json_message.msg_id.clone(), 
                                                        json!(tmp_antenna));
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating antenna: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }
 
    //========================================================================
    // DATABASE OPERATIONS
    //========================================================================

    /**
     * Insert a new Antenna record in the table
     */
     fn insert_db(conn: &SqliteConnection, in_name: &String, in_station_id: &String,
        in_latitude: f64, in_longitude: f64, in_altitude: f64)  -> Result<AntennaDb, diesel::result::Error>
    {            
        let new_antenna = AntennaDb::new_data(in_name, in_station_id,
            in_latitude, in_longitude, in_altitude);

        // Insert into the table
        match diesel::insert_into(t_antenna::table).values(&new_antenna).execute(conn) {
            Ok(_n) => {
                return Ok(new_antenna);
            },
            Err(e) => {
                return Err(e);
            },
        };
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //antenna_dsl.load::<User>(conn).expect("Error loading antennas")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_antenna::table.find(in_id).first::<AntennaDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }
}




