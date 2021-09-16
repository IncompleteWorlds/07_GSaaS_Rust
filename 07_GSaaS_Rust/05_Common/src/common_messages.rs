/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Common Data (messages) structures used in all the API.
 */

//use std::result::Result;

use std::error;
use std::fmt;
use std::time::{SystemTime};

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// HTTP Status code
use actix_web::{http::StatusCode};
//error::ResponseError, HttpResponse, 

pub static REST_JSON_VERSION: &str = "1.0";


// NOT USED

/* 
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
            message:   in_message,
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
*/

// Based on JSONRPC 2.0
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestRequest {
    // Fix value "1.0"
    pub version:                String,
    // Message name. It identifies the type of operation to be executed
    pub msg_code:               String,
    // JWT token. It contains a Claim
    pub authentication_key:     String,
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 String,
    // Current timestamp in Unix time. Seconds since 1/Jan/1970
    pub timestamp:              Value,
    
    
    #[serde(flatten)]
    pub parameters:             Value,
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
            msg_code:               String::new(),
            authentication_key:     String::new(),
            msg_id:                 String::new(),
            //timestamp:              json!( Utc::now().timestamp() ),
            timestamp:              json!( SystemTime::now().elapsed().unwrap() ),
            parameters:             Value::Null,
        }
    }
}

// Based on JSONRPC 2.0
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RestResponse {
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 String,
    
    // Message name. It identifies the type of operation to be executed
    pub msg_code:               String,

    // positive = Warning or info message
    // 0 = No error
    // negative = Error message
    pub status:                 i32,

    // It describes the error
    // It is mandatory on error otherwise empty
    pub detail:                 String, 
    
    // It contains the result of the API call
    // It is mandatory on success
    #[serde(flatten)]
    pub result:                 Value,
}

impl ToString for RestResponse {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}

impl RestResponse {
    
    pub fn new_value(in_msg_code: String, in_msg_id : String, in_value: Value) -> Self {
        let tmp_status : i32 = StatusCode::OK.as_u16() as i32;
        RestResponse {
            msg_id :         in_msg_id,
            msg_code:        String::from(in_msg_code),
            status:          tmp_status,
            detail:          String::new(),
            result:          in_value,
        }
    }
    
    pub fn new_error_msg(in_msg_code: String, in_msg_id : String, in_error_msg: String) -> Self {
        let tmp_status : i32 = StatusCode::BAD_REQUEST.as_u16() as i32;

        RestResponse {
            msg_id :         in_msg_id,
            msg_code:        String::from(in_msg_code),
            status:          tmp_status,
            detail:          in_error_msg,
            result:          Value::Null,
        }
    }
    
    pub fn new_error_id(in_msg_code: String, in_msg_id : String, in_error_code: i32, in_error_msg: String) -> Self {
        RestResponse {
            msg_id :         in_msg_id,
            msg_code:        String::from(in_msg_code),
            status:          in_error_code,
            detail:          in_error_msg,
            result:          Value::Null,
        }
    }
    
    // Create a regular JSON-based response
    pub fn new() -> Self {
        let tmp_status : i32 = StatusCode::OK.as_u16() as i32;

        RestResponse {
            msg_id :         String::new(),
            msg_code:        String::new(),
            status:          tmp_status,
            detail:          String::new(),
            result:          Value::Null,
        }
    }
}


// It represents an internal message between components of the GS
#[derive(Serialize, Deserialize)]
pub struct InternalMessage {
    // The original request
    #[serde(flatten)]
    pub request:         RestRequest,
    
    pub user_id:         String,
    
    // Identifier of the execution. There can be several running in parallel
    pub execution_id:    u32,
}

impl InternalMessage {
    // Prints out the value as JSON string.
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
    
    pub fn new(in_request: &RestRequest, in_user_id: String, in_execution_id : u32) -> Self {
        InternalMessage {
            request:        in_request.clone(),
            user_id:        in_user_id,
            execution_id:   in_execution_id,
        }
    }
}

// {\"response\":{\"msg_id\":\"34456\",\"msg_code_id\":\"test_message\",\"module_id\":0,\"module_instance_id\":1,\"status\":\"Ready\",\"error\":null},\"execution_id\":56,\"wait_flag\":false}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternalResponseMessage {
    // The response
    //#[serde(flatten)]
    pub response:               RestResponse,
    
    // Identifier of the execution. There can be several running in parallel
    pub execution_id:           u32,
    
    // Shall the caller wait for the answer
    pub wait_flag:              bool,
}

impl InternalResponseMessage {
    /// Prints out the value as JSON string.
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
    
    pub fn new_error(in_msg_code: String, in_msg_id: String, in_error_str: &str, in_execution_id: u32) -> Self {  
        InternalResponseMessage {
            response:      RestResponse::new_error_msg(in_msg_code, in_msg_id, String::from(in_error_str)),
            execution_id:  in_execution_id,
            wait_flag:     false,
        }
    }
    
    pub fn new_error_ext(in_msg_code: String, in_msg_id: String, in_error_code : i32, in_error_str: &str, in_execution_id: u32) -> Self {   
        InternalResponseMessage {
            response:      RestResponse::new_error_id(in_msg_code, in_msg_id, in_error_code, String::from(in_error_str) ),
            execution_id:  in_execution_id,
            wait_flag:     false,
        }
    }
    
    pub fn new_value(in_msg_code: String, in_msg_id: String, in_value: Value, in_execution_id: u32) -> Self {
        InternalResponseMessage {
            response:      RestResponse::new_value(in_msg_code, in_msg_id, in_value),
            execution_id:  in_execution_id,
            wait_flag:     false,
        }
    }
    
    pub fn new_wait(in_msg_code: String, in_msg_id: String, in_execution_id: u32) -> Self {
        InternalResponseMessage {
            response:      RestResponse::new_value(in_msg_code, in_msg_id, Value::Null),
            execution_id:  in_execution_id,
            wait_flag:     true,
        }
    }
    
    pub fn new() -> Self {
        InternalResponseMessage {
            response:      RestResponse::new(),
            execution_id:  0,
            wait_flag:     false,
        }
    }
    
}




#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetStatusResponseStruct {
    pub status:   String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetVersionResponseStruct {
    pub version:   String,
}