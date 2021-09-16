/**
* (c) Incomplete Worlds 2021
* Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Ground Stations; CRUD
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
#[table_name="t_ground_station"]
pub struct GroundStationDb
{
    pub id:           String,
    pub name:         String,
    pub owner:        String,
        // Format:  YYYY-MM-DDTHH:MM:SS
    //pub created:        String,
    pub created:      NaiveDateTime,
}


impl GroundStationDb 
{
    /**
     * Create an empty new groundstation
     */
    pub fn new() -> Self 
    {
        GroundStationDb {
            id:             String::new(),
            name:           String::new(),
            owner:          String::new(),
            //created:        Utc::now().to_rfc3339(),
            created:        Utc::now().naive_utc(),
        }
    }

    /**
     * Create an new ground station with the basic information
     */
    fn new_data(in_name: &String, in_owner: &String) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        GroundStationDb {
            id:             new_uuid,
            name:           in_name.clone(),
            owner:          in_owner.clone(),
                //created:        Utc::now().to_rfc3339(),
            created:        Utc::now().naive_utc(),
        }
    }
    
    //========================================================================
    // MESSAGES
    //========================================================================

    /**
     * Create a new ground station.
     * The JSON message shall include the ground station name, ground station description and launch date
     * Document: doc/create_ground_station.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_ground_station",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "name" :                  "Groundstation1",
            "owner" :                 "Institution XYZ"
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "create_ground_station_response",
            "ground_station_id" :     "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "error" :                 null
        }
     */
    pub fn create(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {       
        info!("Create a new ground station: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let create_message : CreateGroundStationStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON CreateGroundStationStruct: {}", e.to_string());
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
        
        if create_message.owner.is_empty() == true {
            let tmp_msg = format!("ERROR: Owner is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // Check if the ground_station already exists
        let groundstation_exist = GroundStationDb::by_id(conn, &create_message.name);
        match groundstation_exist {
            Some(_) => {
                let tmp_msg = format!("This ground_station name is already in use by another ground_station, please enter another name.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
            None => {
            }
        };
    

        // Create and insert the new ground_station into the database
        match GroundStationDb::insert_db(conn, &create_message.name, &create_message.owner) {
            Ok(nm) => {
                let info_msg : String = format!("Created a new ground station with uuid: {}", nm.id);
                info!("{}", info_msg);
                
                let tmp_groundstation = CreateGroundStationResponseStruct { 
                    ground_station_id : nm.id,
                };

                let output = RestResponse::new_value(String::from("create_ground_station_response"), in_json_message.msg_id.clone(), 
                                                        json!(tmp_groundstation));
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating ground station: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }
 
    //========================================================================
    // DATABASE OPERATIONS
    //========================================================================

    /**
     * Insert a new Groundstation record in the table
     */
     fn insert_db(conn: &SqliteConnection, in_name: &String, in_owner: &String)  -> Result<GroundStationDb, diesel::result::Error>
    {            
        let new_groundstation = GroundStationDb::new_data(in_name, in_owner);

        // Insert into the table
        match diesel::insert_into(t_ground_station::table).values(&new_groundstation).execute(conn) {
            Ok(_n) => {
                return Ok(new_groundstation);
            },
            Err(e) => {
                return Err(e);
            },
        };
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //groundstation_dsl.load::<User>(conn).expect("Error loading groundstations")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_ground_station::table.find(in_id).first::<GroundStationDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }
}


