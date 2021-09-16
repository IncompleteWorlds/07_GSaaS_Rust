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

pub static REST_JSON_VERSION: &str = "1.0";


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorData {
    // A Number that indicates the error type that occurred. This MUST be an integer.
    pub code:        i32,

    // A String providing a short description of the error. The message SHOULD be
    // limited to a concise single sentence.
    pub message:      String,
}

impl ErrorData {
    pub fn new(in_code: i32, in_message: String) -> Self {
        Self {
            code:      in_code,
            message:   String::from(in_message),
        }
    }

    pub fn new_str(in_code: i32, in_message: &str) -> Self {
        Self {
            code:      in_code,
            message:   String::from(in_message),
        }
    }

    pub fn std(code: i32) -> Self {
        match code {
            // Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
            -1 => ErrorData::new_str(-32603, "Internal error"),
            // Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
            -32700 => ErrorData::new_str(-32700, "Parse error"),
            // The JSON sent is not a valid Request object.
            -32600 => ErrorData::new_str(-32600, "Invalid Request"),
            // The method does not exist / is not available.
            -32601 => ErrorData::new_str(-32601, "Method not found"),
            // Invalid method parameter(s).
            -32602 => ErrorData::new_str(-32602, "Invalid params"),
            // Internal JSON-RPC error.
            -32603 => ErrorData::new_str(-32603, "Internal error"),
            // The error codes from and including -32768 to -32000 are reserved for pre-defined errors. Any code within
            // this range, but not defined explicitly below is reserved for future use.
            _ => panic!("Undefined pre-defined error codes"),
        }
    }

    /// Prints out the value as JSON string.
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}

impl error::Error for ErrorData {}
impl fmt::Display for ErrorData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.code, self.message)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestRequest {
    // Fix value "1.0"
    pub version:                String,
    // Message name. It identifies the type of operation to be executed
    pub msg_code_id:            String,
    // JWT token. It contains a Claim
    pub authentication_key:     String,
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 String,
    // Current timestamp in Unix time. Seconds since 1/Jan/1970
    pub timestamp:              Value,
    
    
    #[serde(flatten)]
    pub parameters:             Value,
    // Internal parameters
    //#[serde(skip)]
    //pub user_id:                String,
    // Identifier of the execution. There can be several running in parallel
    //#[serde(skip)]
    //pub execution_id:           u32,
}

impl ToString for RestRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}

impl RestRequest {
    // Create a response of type error
    pub fn new() -> Self {
        RestRequest {
            version:                String::from(REST_JSON_VERSION),
            msg_code_id:            String::new(),
            authentication_key:     String::new(),
            msg_id:                 String::new(),
            timestamp:              json!( Utc::now().timestamp() ),
            parameters:             Value::Null,
            //user_id:                String::new(),
            //execution_id:           0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestResponse {
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 String,

    // It contains the result of the API call
    // It is mandatory on success
    #[serde(flatten)]
    pub result:                 Value,

    // It describes the error
    // It is mandatory on error
    pub error:                  Option<ErrorData>,
}

impl ToString for RestResponse {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}

impl RestResponse {
   
    pub fn new_value(in_msg_id : &String, in_value: Value) -> Self {
        RestResponse {
            msg_id :         in_msg_id.clone(),
            result:          in_value.clone(),
            error:           None,
        }
    }

    pub fn new_error_msg(in_msg_id : &String, in_error_msg: String) -> Self {
        RestResponse {
            msg_id :         in_msg_id.clone(),
            result:          Value::Null,
            error:           Some( ErrorData::new(-1, in_error_msg) ),
        }
    }

//     pub fn new_error_id(in_msg_id : &String, in_error_code: i32, in_error_msg: &String) -> Self {
//         RestResponse {
//             msg_id :         String::from(in_msg_id),
//             result:          Value::Null,
//             error:           Some( ErrorData::new(in_error_code, in_error_msg) ),
//         }
//     }

    // Create a regular JSON-based response
    pub fn new(in_response : &InternalResponseMessage) -> Self {
        RestResponse {
            msg_id :         in_response.msg_id.clone(),
            result:          in_response.parameters.clone(),
            error:           in_response.error.clone(),
        }
    }

}


// It represents an internal message between components of the GS
// #[derive(Serialize, Deserialize)]
// pub struct InternalMessage {
//     // The original request
//     #[serde(flatten)]
//     pub request:         RestRequest,
    
//     // Identifier of the execution. There can be several running in parallel
//     pub execution_id:    u32,
// }

// impl InternalMessage {
//     // Prints out the value as JSON string.
//     pub fn to_string(&self) -> String {
//         serde_json::to_string(self).expect("Should never failed")
//     }

//     pub fn new(&mut self, in_request: RestRequest, in_execution_id : u32) -> Self {
//         InternalMessage {
//             request:        in_request.clone(),
//             execution_id:   in_execution_id,
//         }
//     }
// }


#[derive(Serialize, Deserialize, Debug)]
pub struct InternalResponseMessage {
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 String,

    // Identifier of the execution. There can be several running in parallel
    pub execution_id:           u32,

    // Shall the caller wait for the answer
    pub wait_flag:              bool,

    // It contains the response of the API call
    // It is mandatory on success
    #[serde(flatten)]
    pub parameters:             Value,

    // It describes the error
    // It is mandatory on error
    pub error:                  Option<ErrorData>,
}

impl InternalResponseMessage {
    /// Prints out the value as JSON string.
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }

    pub fn new_error(in_msg_id: &str, in_error_str: &str, in_execution_id: u32) -> Self {  
        InternalResponseMessage {
            msg_id:        String::from(in_msg_id),
            execution_id:  in_execution_id,
            wait_flag:     false,
            parameters:    Value::Null,
            error:         Some( ErrorData::new_str(-1, in_error_str) ),
        }
    }

    pub fn new_error_ext(in_msg_id: &str, in_error_code : i32, in_error_str: &str, in_execution_id: u32) -> Self {   
        InternalResponseMessage {
            msg_id:        String::from(in_msg_id),
            execution_id:  in_execution_id,
            wait_flag:     false,
            parameters:    Value::Null,
            error:         Some( ErrorData::new_str(in_error_code, in_error_str) ),
        }
    }

    pub fn new(in_msg_id: &str, in_answer: Value, in_execution_id: u32) -> Self {
        InternalResponseMessage {
            msg_id:        String::from(in_msg_id),
            execution_id:  in_execution_id,
            wait_flag:     false,
            parameters:    in_answer,
            error:         None,
        }
    }

    pub fn new_wait(in_msg_id: &str, in_execution_id: u32) -> Self {
        InternalResponseMessage {
            msg_id:        String::from(in_msg_id),
            execution_id:  in_execution_id,
            wait_flag:     true,
            parameters:    Value::Null,
            error:         None,
        }
    }

    pub fn new_empty() -> Self {
        InternalResponseMessage {
            msg_id:        String::new(),
            execution_id:  0,
            wait_flag:     false,
            parameters:    Value::Null,
            error:         None,
        }
    }

}



// ===================================================

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ErrorStruct
// {
//     // It describes the error
//     // It is mandatory on error
//     pub error:     ErrorData,
// }

// impl ErrorStruct {
//     pub fn new(in_error: ErrorData) -> Self {
//         ErrorStruct {
//             error: in_error,
//         }
//     }

//     // Prints out the value as JSON string.
//     pub fn to_string(&self) -> String {
//         serde_json::to_string(self).expect("Should never failed")
//     }
// }

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
    pub status: String,
}

/**
 * This is a special message. Message for registering a new user
 * It will return the new user id and his authentication key
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterStruct
{
    pub username :     String,
    // Password shall be already hashed with SHA-256
    pub password :     String,
    pub email :        String,

}

/**
 * Response to Register message
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseStruct
{
    pub user_id :               String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginStruct
{
    pub username_email:   String,
    pub password:         String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseStruct
{
    pub user_id:                String,
    pub jwt_token:              String,
}


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






