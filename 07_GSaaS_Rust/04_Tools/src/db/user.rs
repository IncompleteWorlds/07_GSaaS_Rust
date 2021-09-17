/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Users; CRUD, FindBy, 
 *    Login, Logout, Register, Deregister
 */

use std::str;

// JSON serialization
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Log 
use log::{debug, error, info, trace, warn};

// Diesel
//#[macro_use]
use diesel;
use diesel::prelude::*;

// UUID
use uuid::Uuid;

use chrono::{Utc};

// Common functions
use common::http_errors::*;
use common::common_messages::*;
use common::data_structs::user::*;
use common::data_structs::license::*;

// JWT Tokens
use common::claims::*;


// Database
use crate::db::schema::*;

use crate::api_messages::*;



const TOKEN_DURATION_MINS : i64 = 5;

// Note: u32 cannot be used. It has to be i32

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_user"]
pub struct UserDb 
{
    pub id:              String,
    pub username:        String,
    // Hashed password. So, it is not stored in clear
    pub password:        String,
    pub email:           String,
    // License type
    // Demo, Education, Community, Professional
    pub license_id:      String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String,
    // 1 = Logged in, 0 = Logged out
    pub logged:          i32,

    pub role_id:         String,
} 

impl UserDb 
{
    /**
     * Create an empty new user
     */
    pub fn new() -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        UserDb {
            id:             new_uuid,
            username:       String::new(),
            password:       String::new(),
            email:          String::new(),
            license_id:     EnumLicenseType::DemoLicense.to_string(),
            // created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
            logged:         0,
            role_id:        EnumUserRoles::Normal.to_string(),
        }
    }
    
    /**
     * Create an new user with the basic information
     */
    fn new_data(in_username: &String, in_password: &String, 
                    in_email: &String) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        UserDb {
            id:             new_uuid,
            username:       in_username.clone(),
            password:       in_password.clone(),
            email:          in_email.clone(),
            license_id:     EnumLicenseType::DemoLicense.to_string(),
            //created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
            logged:         0,
            role_id:        EnumUserRoles::Normal.to_string(),
        }
    }

    //========================================================================
    // MESSAGES
    //========================================================================

    /**
     * Register a new user.
     * The JSON message shall include the user name, hashed password and the email
     * Document: doc/register.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "register",
            "authentication_key" :    "",
            "user_id" :               "",
            "msg_id" :                "0001",
            "timestamp" :             "",

            "first_ame" :             "my first",
            "last_name" :             "last",
            "role_id" :               "mission_administrator",
            "username" :              "user01",
            "password" :              "aad415a73c4cef1ef94a5c00b2642b571a3e5494536328ad960db61889bd9368",
            "email" :                 "john_doe@someaddress.com",
            "license" :               "Demo"

        Response:
        { 
            "msg_id":"0001",
            "msg_code_id":"error_response",
            "status":  HTTP code,
            "detail":"ERROR: Authentication key not found. IGNORED"
        }

        Roles:
        	

        It returns the new user id
     */
    pub fn register(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, HttpServiceError> 
    {       
        info!("Register a new user: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let register_message : RegisterStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON RegisterStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );

                    return Err(HttpServiceError::InternalServerError(String::from("none"), tmp_msg));
                },
        };

        // Check parameters
        if register_message.username.is_empty() == true {
            let tmp_msg = format!("ERROR: Username is empty. Please fill in");

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }
        
        if register_message.password.is_empty() == true {
            let tmp_msg = format!("ERROR: Password is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );

            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }

        if register_message.email.is_empty() == true {
            let tmp_msg = format!("ERROR: Email is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );

            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }

        // TODO: Apply hash to password
        //let new_password = register_message.password;
        // let hash: String = Sha256::hash((number * BASE).to_string().as_bytes()).hex();

        /*
        // Create a Sha256 object
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(register_message.password);

        // read hash digest and consume hasher
        let new_password = format!("{:x}", hasher.finalize() );
        */

        // Check if the user already exists
        let user_exist = UserDb::by_username(conn, &register_message.username);
        if let Some(_) = user_exist {
            let tmp_msg = format!("This username is already in use by another user, please enter another username.");

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }


        let user_exist = UserDb::by_email(conn, &register_message.email);
        if let Some(_) = user_exist {
            let tmp_msg = format!("This e-mail is already in use by another user, please enter another e-mail.");

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }

        // Create and insert the new user into the database
        match UserDb::insert_db(conn, &register_message.username, &register_message.password, &register_message.email) {
            Ok(nu) => {
                let info_msg : String = format!("Created a new user with uuid: {}", nu.id);
                info!("{}", info_msg);
                
                let tmp_user = RegisterResponseStruct { 
                    user_id : nu.id,
                };

                let output = RestResponse::new_value(String::from("register_response"), in_json_message.msg_id.clone(), 
                                                          json!(tmp_user));
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating user: {}", e);

                error!("{}", error_msg);
                return Err(HttpServiceError::InternalServerError(in_json_message.msg_id.clone(), error_msg));
            },
        };
    }

    /**
     * De-Register a new user.
     * The JSON message shall include the user name, hashed password and the email
     * Document: doc/register.txt
        {
            "version" :               "1.0",
            "msg_code_id" :           "deregister",
            "authentication_key" :    "KKKKKK",
            "user_id" :               "iiiii",
            "msg_id" :                "0001",
            "timestamp" :             ""
        }

        Response:
        { 
            "msg_id":"0001",
            "msg_code_id":"error_response",
            "status":  HTTP code,
            "detail":"ERROR: Authentication key not found. IGNORED"
        }
     */
    pub fn deregister(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, HttpServiceError> 
    {       
        info!("De-register an user: ");

        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.parameters.clone() );
        let deregister_message : DeregisterStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON DeregisterStruct: {}", e.to_string());

                    error!("{}", tmp_msg.as_str() );
                    return Err(HttpServiceError::InternalServerError(String::from("none"), tmp_msg));
                },
        };

        // Check parameters
        if deregister_message.user_id.is_empty() == true {
            let tmp_msg = format!("ERROR: User Id is not provided. Please enter it");
            error!("{}", tmp_msg.as_str() );

            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }

        // Check if the user already exists
        let user_exist = UserDb::by_id(conn, &deregister_message.user_id);
        if user_exist.is_none() == true {
            let tmp_msg = format!("The user does not exist. Nothing is done");

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg));
        }

        // Create and insert the new user into the database
        match UserDb::delete_db(conn, &deregister_message.user_id) {
            Ok(_) => {
                let info_msg : String = format!("User with uuid: {} deleted", deregister_message.user_id);
                info!("{}", info_msg);
                
                let output = RestResponse::new_value(String::from("deregister_response"), in_json_message.msg_id.clone(), 
                Value::Null);
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error deleting user: {}", e);

                error!("{}", error_msg);
                return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), error_msg));
            },
        };
    }

    /**
     * Logout an user
     * Set the logged flat to 0
     */
    pub fn logout(conn: &SqliteConnection, in_json_message: &RestRequest) -> Result<RestResponse, HttpServiceError> {
        info!("logout an user: ");

        // Decode the JSON object
        let in_user : LogoutStruct = match serde_json::from_value(in_json_message.parameters.clone()) {
            Ok(u) => u,
            Err(e) => {
                let error_msg : String = format!("ERROR: Decoding JSON Logout message: {}", e);
                error!("{}", error_msg);

                return Err(HttpServiceError::InternalServerError(String::from("none"), error_msg));
            },
        };

        // Check if the user is logged in (in the DB)
        // First by id
        let tmp_user = UserDb::by_id(conn, &in_user.user_id);

        if tmp_user.is_none() == true {
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), String::from("ERROR: User does not exist")));
        } 

        let read_user : UserDb = tmp_user.unwrap();

        // Check if the user is logged in
        if read_user.logged == 0 {
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), String::from("ERROR: User is not logged in")));
        }

        // Set user as logged
        if let Err(e) = UserDb::set_logged_flag(&conn, &read_user.id, 0) {
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), e.to_string()));
        }

        let output = RestResponse::new_value(String::from("logout_response"), in_json_message.msg_id.clone(), 
                                                  Value::Null);
        Ok(output)
    }

    /**
     * Check the user exists, the password is correct
     * If so, it flags the user as logged
     * Return the user id and the license type
     */
    pub fn login(conn: &SqliteConnection, in_json_message: &RestRequest, in_secret_key: &String) -> Result<RestResponse, HttpServiceError> {
        info!("Login a new user: ");

        // Decode the JSON object
        let in_user : LoginStruct = match serde_json::from_value(in_json_message.parameters.clone()) {
            Ok(u) => u,
            Err(e) => {
                let error_msg : String = format!("ERROR: Decoding JSON Login message: {}", e);
                error!("{}", error_msg);

                return Err(HttpServiceError::InternalServerError(String::from("none"), error_msg));
            },
        };

        // Check if the user already exists (in the DB)
        // First by username
        let mut tmp_user = UserDb::by_username(conn, &in_user.username_email);

        if tmp_user.is_none() == true {
            // Second by email
            tmp_user = match UserDb::by_email(conn, &in_user.username_email) {
                Some(u) => Some(u),
                None => return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), String::from("ERROR: User does not exist"))),
            }
        } 
        let read_user : UserDb = tmp_user.unwrap();
        // println!("Read password {:?}", read_user.password.clone());
        // println!("In password {:?}", in_user.password.clone());

        if read_user.password != in_user.password {
            return Err(HttpServiceError::Unauthorized(in_json_message.msg_id.clone()));
        }
        
        // Check if the user is already logged in
        if read_user.logged == 1 {
            return Err(HttpServiceError::BadRequest(in_json_message.msg_id.clone(), String::from("ERROR: User is already logged in")));
        }

        // Generate JWT Token
        let user_struct = User {
            id:          read_user.id,
            username:    read_user.username,
            password:    read_user.password,
            email:       read_user.email,
            license_id:  read_user.license_id,
            created:     read_user.created,
            role_id:     read_user.role_id,
        };
        let token = match Claims::create_token(&user_struct, TOKEN_DURATION_MINS, 
                                                      in_secret_key) {
            Ok(t) => t,
            Err(_e) => {
                return Err(HttpServiceError::InternalServerError(in_json_message.msg_id.clone(), String::from("ERROR: Unable to generate Token")));
            },
        };

        let login_response : LoginResponseStruct = LoginResponseStruct {
            // Moving data. Not longer used, so it is fine
            user_id :      user_struct.id,
            jwt_token:     token,
            license:       user_struct.license_id,
        };
       
        let output = RestResponse::new_value(String::from("login_response"), in_json_message.msg_id.clone(), 
                                                  json!(login_response));

        // Set user as logged
        if let Err(e) = UserDb::set_logged_flag(&conn, &login_response.user_id, 1) {
            return Err(HttpServiceError::InternalServerError(in_json_message.msg_id.clone(), e.to_string()));
        }
         
        Ok(output)
    }

    
    //========================================================================
    // DATABASE OPERATIONS
    //========================================================================
    /**
     * Insert a new UserDb record in the table
     */
    pub fn insert_db(conn: &SqliteConnection, in_username: &String, in_password: &String,
                    in_email: &String)  -> Result<UserDb, diesel::result::Error>
    {            
        let new_user = UserDb::new_data(in_username, in_password, in_email);

        // Insert into the table
        match diesel::insert_into(t_user::table).values(&new_user).execute(conn) {
            Ok(_n) => {
                return Ok(new_user);
            },
            Err(e) => {
                return Err(e);
            },
        };
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //user_dsl.load::<UserDb>(conn).expect("Error loading users")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_user::table.find(in_id).first::<UserDb>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    pub fn by_username(conn: &SqliteConnection, in_username: &String) -> Option<Self> 
    {
        if in_username.is_empty() == true {
            return None;
        }

        match t_user::table.filter( t_user::username.eq(&in_username) ).first::<UserDb>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    pub fn by_email(conn: &SqliteConnection, in_email: &String) -> Option<Self> 
    {
        if in_email.is_empty() == true {
            return None;
        }

        match t_user::table.filter( t_user::email.eq(&in_email) ).first::<UserDb>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    pub fn delete_db(conn: &SqliteConnection, in_user_id: &String) -> Result<usize, diesel::result::Error> {

        diesel::delete(t_user::table.find(in_user_id)).execute(conn)
    }

    pub fn get_license(conn: &SqliteConnection, in_id: &String) -> String 
    {
        let not_found_error = String::from("User not found");

        if in_id.is_empty() == true {
            return not_found_error;
        }

        match t_user::table.find(in_id).first::<UserDb>(conn) {
            Ok(u) => {
                let license_type = EnumLicenseType::from_string(   u.license_id.as_str() );

                u.license_id.clone()
            },
            Err(_e) => {
                not_found_error
            },
        }
    }

    /**
     * Set the flag that indicates whether an user is logged or not
     */
    fn set_logged_flag(conn: &SqliteConnection, in_id: &String, in_flag: i32) -> Result<(), String>
    {
        let number_rows = diesel::update(t_user::table.find(in_id))
            .set( t_user::logged.eq(in_flag))
            .execute(conn);

        match number_rows {
            Ok(_n) => {
                Ok(())
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }
}
