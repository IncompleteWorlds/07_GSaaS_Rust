/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Data structures used in the FDS API
 */

//use std::result::Result;

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json;

//use std::collections::HashMap;


// This enum represents all HTTP REST REQUEST message that will be received
// from the client
#[derive(Serialize, Deserialize)]
#[serde(tag="msg_code_id")]
pub enum RequestMessage {
    RegisterMessage(RegisterStruct),

}

// This enum represents all HTTP REST RESPONSE message that will be returned
// to the client
#[derive(Serialize, Deserialize)]
#[serde(tag="msg_code_id")]
pub enum ResponseMessage {
    ErrorResponse(ErrorResponseStruct),

    GetVersionMessageResponse(GetVersionResponseStruct),
    
    GetStatusMessageResponse(GetStatusResponseStruct),
    
    RegisterMessageResponse(RegisterResponseStruct),
}

#[derive(Serialize, Deserialize)]
pub struct InternalMessage {
    // Common to all messages
    #[serde(flatten)]
    pub header:       InternalHeaderStruct,

    pub msg:          RequestMessage,
}

#[derive(Serialize, Deserialize)]
pub struct InternalResponseMessage {
    // Common to all messages
    #[serde(flatten)]
    pub header:       InternalHeaderStruct,

    pub msg:          ResponseMessage,
}

// ===================================================

/**
 * Common header to all internal messages
 */
#[derive(Serialize, Deserialize)]
pub struct InternalHeaderStruct {
    pub msg_id:         u32,
    pub execution_id:   u32,
    pub wait_flag:      bool,
    pub error_flag:     bool,
    pub error_msg:      String,
}

// /**
//  * Internal message
//  * The main control loop will send it to the HTTP handler for sharing the task id 
//  * it will have to wait for
//  */
// #[derive(Serialize, Deserialize)]
// pub struct InternalStruct {
//     // Common to all messages
//     #[serde(flatten)]
//     pub header:         InternalHeaderStruct,

//     pub answer:         String,
// }


/**
 * Common to all external API messages
 * --------------------------------------------------------------
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiHeaderStruct {
    // Fix value "1.0"
    pub version:                String,
    pub msg_code_id:            String,
    pub authentication_key:     String,
    pub user_id:                String,
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 u32,
    // Current timestamp in Unix time. Seconds since 1/Jan/1970
    pub timestamp:              String,
}

impl ToString for ApiHeaderStruct {
    fn to_string(&self) -> String {
        let output_buffer : String = format!("Msg Id: {}  Msg Code: {}  Authentication key: {}  User id: {}  Timestamp: {}", 
                                            self.msg_id, 
                                            self.msg_code_id, 
                                            self.authentication_key, 
                                            self.user_id,
                                            self.timestamp);
        
        output_buffer
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponseStruct {
    // Message unique identifier. It will allow to correlate response with the request
    // This value shall be copied into the same field of the response
    pub msg_id:                 u32,
}

impl ToString for ApiResponseStruct {
    fn to_string(&self) -> String {
        let output_buffer : String = format!("Msg Id: {}", 
                                            self.msg_id);

        output_buffer
    }
}

/**
 * Structs definitions
 * --------------------------------------------------------------
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponseStruct
{
    // Common to all responses messages
    #[serde(flatten)]
    pub header:         ApiResponseStruct,

    pub error_flag:     bool,
    pub error_msg:      String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionResponseStruct
{
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStatusResponseStruct
{
    pub status: String,
}

/**
 * This is a special message. Message for registering a new user
 * It will return the new user id and his authentication key
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterStruct
{
    // Common to all messages
    #[serde(flatten)]
    pub header:        ApiHeaderStruct,

    pub username :     String,
    // Password shall be already hashed with SHA-256
    pub password :     String,
    pub email :        String,

}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseStruct
{
    // Common to all responses messages
    #[serde(flatten)]
    pub header:                 ApiResponseStruct,

    pub user_id :               String,
    pub authentication_key :    String,
}



/*
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMessageAnswer {
    pub error_flag:  bool,
    pub msg_buffer:  String,
}

impl ToString for ApiMessageAnswer {
    fn to_string(&self) -> String {
        let output_buffer = format!("Error flag: {} \n Message: {}", self.error_flag, self.msg_buffer);

        output_buffer
    }
}
*/


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
pub struct GetStatusMessage {
    #[serde(flatten)]
    pub header: ApiMessageHeader,

    pub user_id: String,
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

/**
 * It creates a InternalMessageResponse JSON with the error string
 * Receiver does not have to wait
 */
pub fn build_error_str_json(in_msg_id: u32, in_error_str: &str) -> String {   
    let tmp = ErrorResponseStruct {
        header : ApiResponseStruct {
            msg_id:     in_msg_id,
        },
        error_flag:     true,
        error_msg:      String::from(in_error_str),
    };
    let outputError = ResponseMessage::ErrorResponse(tmp);

    let resp_json_message = serde_json::to_string(&outputError);

    return resp_json_message.unwrap();
}

/**
 * It creates a InternalMessageResponse JSON with the answer
 * Receiver does not have to wait
 */
pub fn build_answer_str_json(in_msg_id: u32, in_answer: &str) -> String {
    let tmp = ErrorResponseStruct {
        header : ApiResponseStruct {
            msg_id:     in_msg_id,
        },
        error_flag:     false,
        error_msg:      String::from(in_answer),
    };
    let outputError = ResponseMessage::ErrorResponse(tmp);

    let resp_json_message = serde_json::to_string(&outputError);

    return resp_json_message.unwrap();
}



/* *
 * It creates a InternalMessage JSON for indicating to the receiver that 
 * it must wait for the task (with in_execution_id) to complete
 */
// pub fn build_int_wait_answer_str_json(in_execution_id: u32, in_message: ResponseMessage) -> String {
//     let tmp = InternalMessageResponse {
//         header : InternalHeaderStruct {
//             error_flag:     false,
//             error_msg:      String::new(),
//             execution_id:   in_execution_id,
//             wait_flag:      true,
//             msg_id:         0,
//         },
//         msg:  in_message ,
//     };

//     let resp_json_message = serde_json::to_string(&tmp);

//     return resp_json_message.unwrap();
// }






