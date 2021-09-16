/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Mission; CRUD, FindBy, 
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



// Note: u32 cannot be used. It has to be i32

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_mission"]
pub struct MissionDb
{
    pub id:              String,
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String, 
} 

impl MissionDb 
{
    /**
     * Create an empty new mission
     */
    pub fn new() -> Self 
    {
        MissionDb {
            id:             String::new(),
            name:           String::new(),
            description:    String::new(),
            // created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
        }
    }

    /**
     * Create an new mission with the basic information
     */
    fn new_data(in_name: &String, in_description: &String, 
                in_launch_date: &String) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        MissionDb {
            id:             new_uuid,
            name:           in_name.clone(),
            description:    in_description.clone(),
            //created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
        }
    }
    
    //========================================================================
    // MESSAGES
    //========================================================================
    
    /**
     * Create a new mission.
     * The JSON message shall include the mission name, mission description and launch date
     * Document: doc/create_mission.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_mission",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "name" :                  "Satellite 1",
            "description" :           "The first one",
            "launch_date" :           "2005-11-30"
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "create_mission_response",
            "mission_id" :            "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "error" :                 null
        }
     */
    pub fn create(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {       
        info!("Create a new mission: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let create_message : CreateMissionStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON CreateMissionStruct: {}", e.to_string());
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

        // Check if the Mission already exists
        let mission_exist = MissionDb::by_name(conn, &create_message.name);
        match mission_exist {
            Some(_) => {
                let tmp_msg = format!("This mission name is already in use by another mission, please enter another name.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
            None => {
            }
        };
       

        // Create and insert the new mission into the database
        match MissionDb::insert_db(conn, &create_message.name, &create_message.description,
            &create_message.launch_date) {
            Ok(nm) => {
                let info_msg : String = format!("Created a new mission with uuid: {}", nm.id);
                info!("{}", info_msg);
                
                let tmp_mission = CreateMissionReponseStruct { 
                    mission_id : nm.id,
                };

                let output = RestResponse::new_value(String::from("create_mission_response"), in_json_message.msg_id.clone(), 
                                                          json!(tmp_mission));
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating mission: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }

    /**
     * Delete a mission.
     * The JSON message shall include the mission id
     * Document: doc/delete_mission.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "create_mission",
            "authentication_key" :    "XXXYYYZZZ",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "msg_id" :                "0001",
            "timestamp" :             "2021-06-17T18:35:45",

            "mission_id" :            "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
        }

        Response:
        {
            "msg_id" :                "0001",
            "msg_code_id" :           "delete_mission_response",
            "error" :                 null
        }
     */
    pub fn delete(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, String> 
    {
        info!("Delete a mission: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let delete_message : DeleteMissionStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON DeleteMissionStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // Check parameters
        if delete_message.mission_id.is_empty() == true {
            let tmp_msg = format!("ERROR: Name is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // Check if the Mission already exists
        let mission_exist = MissionDb::by_name(conn, &delete_message.mission_id);
        match mission_exist {
            Some(_) => {
            }
            None => {
                let tmp_msg = format!("The mission does not exist");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
            }
        };

        // Delete the Mission record
        match MissionDb::delete_db(conn, &delete_message.mission_id) {
            Ok(_) => {
                let info_msg : String = format!("Mission with id: {} deleted", delete_message.mission_id);
                info!("{}", info_msg);
                
                let output = RestResponse::new_value(String::from("delete_mission_response"), in_json_message.msg_id.clone(), 
                Value::Null);
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error deleting Mission: {}", e);
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
        info!("Read mission data: ");

        info!("TODO");

        let output = RestResponse::new_value(String::from("read_mission_response"), in_json_message.msg_id.clone(), 
                Value::Null);
        return Ok(output);
    }
    
    
    //========================================================================
    // DATABASE OPERATIONS
    //========================================================================

    /**
     * Insert a new Mission record in the table
     */
    fn insert_db(conn: &SqliteConnection, in_name: &String, in_description: &String, 
        in_launch_date: &String)  -> Result<MissionDb, diesel::result::Error>
    {            
        let new_mission = MissionDb::new_data(in_name, in_description, in_launch_date);

        // Insert into the table
        match diesel::insert_into(t_mission::table).values(&new_mission).execute(conn) {
            Ok(_n) => {
                return Ok(new_mission);
            },
            Err(e) => {
                return Err(e);
            },
        };
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //mission_dsl.load::<User>(conn).expect("Error loading missions")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_mission::table.find(in_id).first::<MissionDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }

    pub fn by_name(conn: &SqliteConnection, in_name: &String) -> Option<Self> 
    {
        if in_name.is_empty() == true {
            return None;
        }

        match t_mission::table.filter( t_mission::name.eq(&in_name) ).first::<MissionDb>(conn) {
            Ok(m) => Some(m),
            Err(_e) => None,
        }
    }
    
    pub fn delete_db(conn: &SqliteConnection, in_id: &String) -> Result<usize, diesel::result::Error> 
    {
        diesel::delete(t_mission::table.find(in_id)).execute(conn)
    }
}




