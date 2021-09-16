/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Data structures used in the TM Decoder
 */

use std::result::Result;

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json;

//use std::collections::HashMap;




//#[derive(Serialize, Deserialize)]
// pub struct Parameter {
//     pub key:    String,
//     pub value:  String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMessage {
    pub msg_code_id:            String,
    pub authentication_key:     String,
    // pub msg_json:               String,
    // pub parameters:             HashMap<String, String>,
    // pub second_lev_parameters:  HashMap<String, Vec<Parameter>>,
}

impl ToString for ApiMessage {
    fn to_string(&self) -> String {
        let mut output_buffer : String = format!("Msg Code: {}  ", self.msg_code_id);
        
        //output_buffer.push_str(self.msg_json.as_str());

        /*
        for aParameter in self.parameters {
            let tmp_parameter : String = format!("{}: {}  ", aParameter.0, aParameter.1);

            output_buffer.push_str(tmp_parameter.as_str());
        }

        for secParameter in self.second_lev_parameters {
            let tmp_parameter : String = format!("{}: ", secParameter.0);
            output_buffer.push_str(tmp_parameter.as_str());

            for otherParameter in secParameter.1 {
                let tmp_parameter : String = format!("{}: {}  ", otherParameter.key, otherParameter.value);

                output_buffer.push_str(tmp_parameter.as_str());
            }
        }
        */

        return output_buffer;
    }
}

/*
 {
     "msg_code_id" : "012345",
     "authentication_key" : "xxxuuuaaaxxxa",
     "parameters" : [ 
         "key" : "value",
     ]
 }

*/

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMessageAnswer {
    // For internal use only
    pub msg_result:  std::result::Result<(), String>,
    // Output text message
    pub msg_buffer:  String,
}

impl ToString for ApiMessageAnswer {
    fn to_string(&self) -> String {
        let mut output_buffer : String;
        
        match &self.msg_result {
            Ok(_k) => { output_buffer = String::from("Ok") },
            Err(e) => { output_buffer = e.to_string() },
        };

        output_buffer.push_str(" ");
        output_buffer.push_str( self.msg_buffer.as_str() );
        return output_buffer;
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct OrbPropagationMessage {
    pub header: ApiMessage,

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
    pub header: ApiMessage,

    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStatusMessageResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionMessage {
    pub header: ApiMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionMessageResponse {
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExitMessage {
    pub header: ApiMessage,

    pub exit_code: String,
}







pub fn build_api_message(in_code_id: &String, in_auth_key: &String, in_msg_json: &String) -> ApiMessage {
    ApiMessage {
        msg_code_id :        in_code_id.clone(),
        authentication_key:  in_auth_key.clone(),
        //msg_json :           in_msg_json.clone(),
    }
}


pub fn build_api_message_str(in_code_id: &str, in_auth_key: &str, in_msg_json: &str) -> ApiMessage {
    ApiMessage {
        msg_code_id :        String::from(in_code_id),
        authentication_key:  String::from(in_auth_key),
        //msg_json :           String::from(in_msg_json),
    }
}

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


pub fn build_api_answer_str_json(in_error_flag : bool, in_error_str: &str, in_msg_json: &str) -> String {
    let mut tmp = ApiMessageAnswer {
        msg_result :        Ok(()),
        msg_buffer :        String::from(in_msg_json),
    };

    if in_error_flag == true {
        tmp.msg_result = Err( String::from(in_error_str) );
    }

    let resp_json_message = serde_json::to_string(&tmp);

    return resp_json_message.unwrap();
}






