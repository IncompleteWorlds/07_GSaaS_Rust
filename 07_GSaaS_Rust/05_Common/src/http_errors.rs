/**
* (c) Incomplete Worlds 2021
* Alberto Fernandez (ajfg)
*
* GS as a Service
* 
* Service errors definition
*/

use actix_web::{error::ResponseError, HttpResponse, http::StatusCode};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use std::convert::From;
use uuid::Error as ParseError;

use crate::common_messages::*;



#[derive(Debug, Display)]
pub enum HttpServiceError {
    // msg_id, error message
    #[display(fmt = "Internal Server Error")]
    InternalServerError(String, String),

    // msg_id, error message
    #[display(fmt = "BadRequest: {} {}", _0, _1)]
    BadRequest(String, String),

    // msg_id
    #[display(fmt = "Unauthorized")]
    Unauthorized(String),

    // msg_id, error message
    #[display(fmt = "Forbidden access: {} {}", _0, _1)]
    Forbidden(String, String),

    // msg_id, error message
    #[display(fmt = "Resource not found: {} {}", _0, _1)]
    NotFound(String, String),
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for HttpServiceError {
    fn error_response(&self) -> HttpResponse {

        match self {
            HttpServiceError::InternalServerError(in_msg_id, in_error_msg) => {
                let tmp_code : u16 = (StatusCode::INTERNAL_SERVER_ERROR).into();

                let response = RestResponse::new_error_id(String::from("InternalServerError"), in_msg_id.to_string(), 
                                            tmp_code as i32, in_error_msg.to_string());

                HttpResponse::InternalServerError().content_type("application/json")
                                                   .json(response)
            },


            HttpServiceError::BadRequest(ref in_msg_id, in_error_msg ) => {
                let tmp_code : u16 = (StatusCode::BAD_REQUEST).into();

                let response = RestResponse::new_error_id(String::from("BadRequest"), in_msg_id.to_string(), 
                                            tmp_code as i32, in_error_msg.to_string());

                HttpResponse::BadRequest().content_type("application/json")
                                         .json(response)
            },

            HttpServiceError::Unauthorized(ref in_msg_id) => {
                //HttpResponse::Unauthorized().json("Unauthorized"),
                let tmp_code : u16 = (StatusCode::UNAUTHORIZED).into();

                let response = RestResponse::new_error_id(String::from("Unauthorized"), in_msg_id.to_string(), 
                                            tmp_code as i32, String::from("Unauthorized access"));

                HttpResponse::Unauthorized().content_type("application/json")
                                            .json(response)
            },

            HttpServiceError::Forbidden(ref in_msg_id, in_error_msg ) => {
                let tmp_code : u16 = (StatusCode::FORBIDDEN).into();

                let response = RestResponse::new_error_id(String::from("Forbidden"), in_msg_id.to_string(), 
                                            tmp_code as i32, in_error_msg.to_string());

                HttpResponse::Forbidden().content_type("application/json")
                                         .json(response)
            },

            HttpServiceError::NotFound(ref in_msg_id, in_error_msg ) => {
                let tmp_code : u16 = (StatusCode::NOT_FOUND).into();

                let response = RestResponse::new_error_id(String::from("Not Found"), in_msg_id.to_string(), 
                                            tmp_code as i32, in_error_msg.to_string());

                HttpResponse::NotFound().content_type("application/json")
                                        .json(response)
            },
        }
    }
}

// we can return early in our handlers if UUID provided by the user is not valid
// and provide a custom message
impl From<ParseError> for HttpServiceError {
    fn from(_: ParseError) -> HttpServiceError {
        HttpServiceError::BadRequest(String::from("unknown"), "Invalid UUID".into())
    }
}

impl From<DBError> for HttpServiceError {
    fn from(error: DBError) -> HttpServiceError {
        // Right now we just care about UniqueViolation from diesel
        // But this would be helpful to easily map errors as our app grows
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return HttpServiceError::BadRequest(String::from("unknown"), message);
                }

                HttpServiceError::InternalServerError( String::from("unknown"), String::from("unknown") )
            },

            _ => HttpServiceError::InternalServerError( String::from("unknown"), String::from("unknown") ),
        }
    }
}
