/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Data structures used in the FDS API
 */

//use std::result::Result;

use std::error;
use std::fmt;

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use chrono::{DateTime, Utc};

//use std::collections::HashMap;



/**
 * Structs definitions
 * --------------------------------------------------------------
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionResponseStruct
{
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStatusResponseStruct {
    pub status:               String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetModuleStatusResponseStruct {
    pub module_id:            u32,
    pub module_instance_id:   u32,
    pub status:               String,
}








// /**
//  * This is a special message. Message for registering a new user
//  * It will return the new user id and his authentication key
//  */
// #[derive(Serialize, Deserialize, Debug)]
// pub struct RegisterStruct
// {
//     pub username :     String,
//     // Password shall be already hashed with SHA-256
//     pub password :     String,
//     pub email :        String,

// }

// /**
//  * Response to Register message
//  */
// #[derive(Serialize, Deserialize, Debug)]
// pub struct RegisterResponseStruct
// {
//     pub user_id :               String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct LoginStruct
// {
//     pub username_email:   String,
//     pub password:         String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct LoginResponseStruct
// {
//     pub user_id:                String,
//     pub jwt_token:              String,
// }


/*

#[derive(Serialize, Deserialize, Debug)]
pub struct OrbPropagationMessage {
    // Common to all messages
    #[serde(flatten)]
    pub header: ApiMessageHeader,

    pub mission_id: String,
    pub satellite_id: String,
    // NOTE: There is no desarialization of DateTime, so, we
    // store it as a String in
    //  RFC 3339 and ISO 8601 date and time string
    // yyyy-MM-ddThh:mm::ss.sss   UTC
    pub start_time: String,
    pub stop_time: String,
    pub step: u32, // seconds
    pub position:  (f64, f64, f64),   // meters
    pub velocity:  (f64, f64, f64),   // meters / second
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrbPoint {
    // Date time in RFC3339
    pub position_time: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrbPropagationMessageResponse {
    // Common to all messages
    #[serde(flatten)]
    pub header: ApiMessageHeader,

    pub mission_id: String,
    pub satellite_id: String,
    // NOTE: There is no desarialization of DateTime, so, we
    // store it as a String in
    //  RFC 3339 and ISO 8601 date and time string
    pub start_time: String,
    pub end_time: String,
    pub step: u32, // seconds
    pub number_points: u32,
    pub list_points: Vec<OrbPoint>,
}







#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionMessage {
    #[serde(flatten)]
    pub header: ApiMessageHeader,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ExitMessage {
    #[serde(flatten)]
    pub header: ApiMessageHeader,

    pub exit_code: String,
}
*/




/*

pub fn build_api_message(in_code_id: &String, in_auth_key: &String, in_msg_json: &String) -> ApiMessageHeader {
    ApiMessageHeader {
        msg_code_id :        in_code_id.clone(),
        authentication_key:  in_auth_key.clone(),
        //msg_json :           in_msg_json.clone(),
    }
}


pub fn build_api_message_str(in_code_id: &str, in_auth_key: &str, in_msg_json: &str) -> ApiMessageHeader {
    ApiMessageHeader {
        msg_code_id :        String::from(in_code_id),
        authentication_key:  String::from(in_auth_key),
        //msg_json :           String::from(in_msg_json),
    }
}
*/

/*
pub fn build_api_answer(in_error_flag : bool, in_error_str: &String, in_msg_json: &String) -> ApiMessageAnswer {
    if in_error_flag == true {
        ApiMessageAnswer {
            msg_result :        Err(in_error_str.clone()),
            msg_buffer :        in_msg_json.clone(),
        }

    } else {
        ApiMessageAnswer {
            msg_result :        Ok(()),
            msg_buffer :        in_msg_json.clone(),
        }
    }
}

pub fn build_api_answer_str(in_error_flag : bool, in_error_str: &str, in_msg_json: &str) -> ApiMessageAnswer {
    if in_error_flag == true {
        ApiMessageAnswer {
            msg_result :        Err(String::from(in_error_str)),
            msg_buffer :        String::from(in_msg_json),
        }

    } else {
        ApiMessageAnswer {
            msg_result :        Ok(()),
            msg_buffer :        String::from(in_msg_json),
        }
    }
}
*/

/*
pub fn build_api_answer_str_json(in_error_flag : bool, in_error_str: &str, in_msg_json: &str) -> String {
    let mut tmp = ApiMessageAnswer {
        error_flag :        in_error_flag,
        msg_buffer :        String::from(in_msg_json),
    };

    if in_error_flag == true {
        tmp.msg_buffer = String::from(in_error_str);
    }

    let resp_json_message = serde_json::to_string(&tmp);

    return resp_json_message.unwrap();
}
*/


// ===================================================================
// ===================================================================
// ===================================================================

/* *
 * It creates a InternalMessageResponse JSON with the error string
 * Receiver does not have to wait
 */
// pub fn build_error_str_json(in_msg_id: &str, in_error_str: &str, in_execution_id: u32) -> String {  
//     let output = InternalResponseMessage {
//         msg_id:        json!(in_msg_id),
//         execution_id:  in_execution_id,
//         wait_flag:     false,
//         response:      Value::Null,
//         error:         Some( ErrorData::new(-1, in_error_str) ),
//     };

//     return output.to_string();    
// }

/* *
 * It creates a InternalMessageResponse JSON with the error string
 * Receiver does not have to wait
 */
// pub fn build_error_str_json_ext(in_msg_id: &str, in_error_code : i32, in_error_str: &str, in_execution_id: u32) -> String {   
//     let output = InternalResponseMessage {
//         msg_id:        json!(in_msg_id),
//         execution_id:  in_execution_id,
//         wait_flag:     false,
//         response:      Value::Null,
//         error:         Some( ErrorData::new(in_error_code, in_error_str) ),
//     };

//     return output.to_string();     
// }

/* *
 * It creates a InternalMessageResponse JSON with the answer
 * Receiver does not have to wait
 */
// pub fn build_answer_str_json(in_msg_id: &str, in_answer: Value, in_execution_id: u32) -> String {
//     let output = InternalResponseMessage {
//         msg_id:        json!(in_msg_id),
//         execution_id:  in_execution_id,
//         wait_flag:     false,
//         response:      in_answer,
//         error:         None,
//     };

//     return output.to_string(); 
// }

/* *
 * It creates a InternalMessage JSON for indicating to the receiver that 
 * it must wait for the task (with in_execution_id) to complete
 */
// pub fn build_wait_answer_str_json(in_msg_id: &str, in_execution_id: u32) -> String 
// {
//     let output = InternalResponseMessage {
//         msg_id:        json!(in_msg_id),
//         execution_id:  in_execution_id,
//         wait_flag:     true,
//         response:      Value::Null,
//         error:         None,
//     };

//     return output.to_string(); 
// }






