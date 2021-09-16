/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Satellite; CRUD
 */

 // JSON serialization
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono::format::ParseError;

// Log 
use log::{debug, error, info, trace, warn};

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
#[table_name="t_satellite"]
pub struct SatelliteDb
{
    pub id:              String,
    pub mission_id:      String,
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub launch_date:     Option<String>, 
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String, 
}

impl SatelliteDb
{
    /**
     * Create an empty new mission
     */
    pub fn new() -> Self 
    {
        SatelliteDb {
            id:             String::new(),
            mission_id:     String::new(),
            name:           String::new(),
            description:    String::new(),
            launch_date:    None,
            // created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
        }
    }
 
    /**
     * Create an new mission with the basic information
     */
    fn new_data(in_mission: &String, in_name: &String, in_description: &String, 
                in_launch_date: &String) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();
 
        SatelliteDb {
            id:             new_uuid,
            mission_id:     in_mission.clone(),
            name:           in_name.clone(),
            description:    in_description.clone(),
            launch_date:    Some( in_launch_date.clone() ),
            //created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
        }
    }

    //========================================================================
    // MESSAGES
    //========================================================================
    
    /**
     * Create a new satellite.
     * The JSON message shall include the mission id, satellite name, description
     * and launch date
     * Document: doc/create_satellite.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_satellite",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "mission_id" :            "012345",
            "name" :                  "Satellite 1",
            "description" :           "The first one",
            "launch_date" :           "2005-11-30"
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "create_satellite_response",
            "satellite_id" :          "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "error" :                 null
        }
     */
    pub fn create(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {       
        info!("Create a new satellite: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let create_message : CreateSatelliteStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON CreateSatelliteStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // Check parameters
        if create_message.mission_id.is_empty() == true {
            let tmp_msg = format!("ERROR: Mission Id is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }
        
        if create_message.name.is_empty() == true {
            let tmp_msg = format!("ERROR: Name is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        if create_message.description.is_empty() == true {
            let tmp_msg = format!("ERROR: Description is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        if create_message.launch_date.is_empty() == true {
            let tmp_msg = format!("ERROR: Launch Date is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // Check if the Satellite already exists
        let satellite_exist = SatelliteDb::by_name(conn, &create_message.name);
        match satellite_exist {
            Some(_) => {
                let tmp_msg = format!("This satellite name is already in use by another satellite, please enter another name.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
            None => {
            }
        };
    

        // Create and insert the new satellite into the database
        match SatelliteDb::insert_db(conn, &create_message.mission_id, &create_message.name, 
            &create_message.description, &create_message.launch_date) {
            Ok(nm) => {
                let info_msg : String = format!("Created a new satellite with uuid: {}", nm.id);
                info!("{}", info_msg);
                
                let tmp_satellite = CreateSatelliteReponseStruct { 
                    satellite_id : nm.id,
                };

                let output = RestResponse::new_value(String::from("create_satellite_response"), in_json_message.msg_id.clone(), 
                                                        json!(tmp_satellite));
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating satellite: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }

    /**
     * Delete a satellite.
     * The JSON message shall include the satellite id
     * Document: doc/delete_satellite.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_satellite",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "satellite_id" :            "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "delete_satellite_response",
            "error" :                 null
        }
     */
    pub fn delete(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {
        info!("Delete a satellite: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let delete_message : DeleteSatelliteStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON DeleteSatelliteStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // Check parameters
        if delete_message.satellite_id.is_empty() == true {
            let tmp_msg = format!("ERROR: Name is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // Check if the Satellite already exists
        let satellite_exist = SatelliteDb::by_name(conn, &delete_message.satellite_id);
        match satellite_exist {
            Some(_) => {
            }
            None => {
                let tmp_msg = format!("The satellite does not exist");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
        };

        // Delete the Satellite record
        match SatelliteDb::delete_db(conn, &delete_message.satellite_id) {
            Ok(_) => {
                let info_msg : String = format!("Satellite with id: {} deleted", delete_message.satellite_id);
                info!("{}", info_msg);
                
                let output = RestResponse::new_value(String::from("delete_satellite_response"), in_json_message.msg_id.clone(), 
                Value::Null);
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error deleting Satellite: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }

    /**
    *
    */
    pub fn read(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {
        info!("Read satellite data: ");

        info!("TODO");

        let output = RestResponse::new_value(String::from("read_satellite_response"), in_json_message.msg_id.clone(), 
                Value::Null);
        return Ok(output);
    }

    //========================================================================
    // DATABASE OPERATIONS
    //========================================================================

    /**
     * Insert a new Satellite record in the table
     */
     fn insert_db(conn: &SqliteConnection, in_mission: &String, in_name: &String, 
        in_description: &String, in_launch_date: &String)  -> Result<SatelliteDb, diesel::result::Error>
    {            
        let new_satellite = SatelliteDb::new_data(in_mission, in_name, in_description, in_launch_date);

        // Insert into the table
        match diesel::insert_into(t_satellite::table).values(&new_satellite).execute(conn) {
            Ok(_n) => {
                return Ok(new_satellite);
            },
            Err(e) => {
                return Err(e);
            },
        };
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //satellite_dsl.load::<User>(conn).expect("Error loading satellites")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_satellite::table.find(in_id).first::<SatelliteDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }

    pub fn by_name(conn: &SqliteConnection, in_name: &String) -> Option<Self> 
    {
        if in_name.is_empty() == true {
            return None;
        }

        match t_satellite::table.filter( t_satellite::name.eq(&in_name) ).first::<SatelliteDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }
    
    pub fn delete_db(conn: &SqliteConnection, in_id: &String) -> Result<usize, diesel::result::Error> 
    {
        diesel::delete(t_satellite::table.find(in_id)).execute(conn)
    }
}