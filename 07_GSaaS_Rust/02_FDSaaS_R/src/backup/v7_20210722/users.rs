/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Users; CRUD
 */

use std::str;
use rand;
//use rand::thread_rng;
use rand::Rng;

// JSON serialization
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Log 
use log::{debug, error, info, trace, warn};

// Diesel
#[macro_use]
use diesel;
use diesel::prelude::*;

// UUID
use uuid::Uuid;

// SHA 256
use sha2::{Sha256, Digest};
// use chrono::{DateTime, Duration, Utc};
// use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

// Database
use crate::schema::*;
//use crate::schema::t_user::dsl::*;
use crate::fds_messages::*;

// JWT Tokens
use crate::claims::*;

use crate::config_fds::{encode_hex};


const TOKEN_DURATION_MINS : i64 = 5; 

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EnumLicenseType 
{
    DemoLicense,
    EducationLicense,
    CommunityLicense,
    ProfessionalLicense,
}

impl EnumLicenseType 
{
    pub fn to_string(&self) -> String 
    {
        match *self 
        {
            EnumLicenseType::DemoLicense             => String::from("Demo"),
            EnumLicenseType::EducationLicense        => String::from("Education"),
            EnumLicenseType::CommunityLicense        => String::from("Community"),
            EnumLicenseType::ProfessionalLicense     => String::from("Professional"),
        }
    }

    pub fn from_string(in_license: &str) -> Self 
    {
        match in_license {
            "Demo"           => EnumLicenseType::DemoLicense,
            "Education"      => EnumLicenseType::EducationLicense,
            "Community"      => EnumLicenseType::CommunityLicense,
            "Professional"   => EnumLicenseType::ProfessionalLicense,
            _                => EnumLicenseType::DemoLicense,
        }
    }
}

// Note: u32 cannot be used. It has to be i32

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_user"]
pub struct User 
{
    pub id:              String,
    pub username:        String,
    // Hashed password. So, it is not stored in clear
    pub password:        String,
    pub email:           String,
    // License type
    // Demo, Education, Community, Professional
    pub license_id:      String,
    pub created:         chrono::NaiveDateTime,
} 

impl User 
{
    /**
     * Create an empty new user
     */
    pub fn new() -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        User {
            id:             new_uuid,
            username:       String::new(),
            password:       String::new(),
            email:          String::new(),
            license_id:     EnumLicenseType::DemoLicense.to_string(),
            created:        chrono::Local::now().naive_local(),
        }
    }

    /**
     * Create an new user with the basic information
     */
    pub fn new_data(in_username: &String, in_password: &String, 
                    in_email: &String, in_key: &String) -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        User {
            id:             new_uuid,
            username:       in_username.clone(),
            password:       in_password.clone(),
            email:          in_email.clone(),
            license_id:     EnumLicenseType::DemoLicense.to_string(),
            created:        chrono::Local::now().naive_local(),
        }
    }


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

            "username" :              "john_doe",
            "password" :              "9Sec$reZATpwd0$",
            "email" :                 "john_doe@someaddress.com",
            "license" :               "Demo"
        }

        Response:
        {
            "msg_id" :                "0001",
            "user_id" :               "0fc1c0e1-878a-4562-ba81-86e20b9b07ab",
            "error" :                 null
        }
     */
    pub fn register(conn: &SqliteConnection, in_json_message: &InternalMessage) -> Result<InternalResponseMessage, String> 
    {       
        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.request.parameters.clone() );
        let register_message : RegisterStruct = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON RegisterStruct: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // Check parameters
        if register_message.username.is_empty() == true {
            let tmp_msg = format!("ERROR: Username is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }
        
        if register_message.password.is_empty() == true {
            let tmp_msg = format!("ERROR: Password is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        if register_message.email.is_empty() == true {
            let tmp_msg = format!("ERROR: Email is empty. Please fill in");
            error!("{}", tmp_msg.as_str() );
            return   Err(tmp_msg);
        }

        // TODO: Apply hash to password
        //let new_password = register_message.password;
        // let hash: String = Sha256::hash((number * BASE).to_string().as_bytes()).hex();


        // Create a Sha256 object
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(register_message.password);

        // read hash digest and consume hasher
        let new_password = format!("{:x}", hasher.finalize() );

        // FIXME: Password shall be already hashed

        // Check if the user already exists
        let user_exist = User::by_username(conn, &register_message.username);
        match user_exist {
            Some(_) => {
                let tmp_msg = format!("This username is already in use by another user, please enter another username.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
                }
            None => {
            }
        };
        let user_exist = User::by_email(conn, &register_message.email);
        match user_exist {
            Some(_) => {
                let tmp_msg = format!("This e-mail is already in use by another user, please enter another e-mail.");
                error!("{}", tmp_msg.as_str() );
                return   Err(tmp_msg);
                }
            None => {
            }
        };

        // Generate random secret key
        let mut tmp_secret_key = [0u8; 16];
        let mut rng = rand::thread_rng();

        for i in 0..16 {
            let x : u8 = rng.gen_range(0..255);

            tmp_secret_key[i] = x;
        }
        let new_secret_key = encode_hex(&tmp_secret_key);

        // Create and insert the new user into the database
        match User::create(conn, &register_message.username, &new_password, &register_message.email, &new_secret_key) {
            Ok(nu) => {
                let info_msg : String = format!("Created a new user with uuid: {}", nu.id);
                info!("{}", info_msg);
                
                let tmp_user = RegisterResponseStruct { 
                    user_id : nu.id,
                };

                let output = InternalResponseMessage::new_value("register_response", in_json_message.request.msg_id.clone(), 
                                                          json!(tmp_user), 0);
                return Ok(output);
            },
            Err(e) => {
                let error_msg : String = format!("Error creating user: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };
    }

    /**
     * Insert a new User record in the table
     */
    pub fn create(conn: &SqliteConnection, in_username: &String, in_password: &String,
                    in_email: &String, in_key: &String)  -> Result<User, diesel::result::Error>
    {            
        let new_user = User::new_data(in_username, in_password, in_email, in_key);

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
        //user_dsl.load::<User>(conn).expect("Error loading users")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if in_id.is_empty() == true {
            return None;
        }

        match t_user::table.find(in_id).first::<User>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    pub fn by_username(conn: &SqliteConnection, in_username: &String) -> Option<Self> 
    {
        if in_username.is_empty() == true {
            return None;
        }

        match t_user::table.filter( t_user::username.eq(&in_username) ).first::<User>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    pub fn by_email(conn: &SqliteConnection, in_email: &String) -> Option<Self> 
    {
        if in_email.is_empty() == true {
            return None;
        }

        match t_user::table.filter( t_user::email.eq(&in_email) ).first::<User>(conn) {
            Ok(u) => Some(u),
            Err(_e) => None,
        }
    }

    
    pub fn logout(_conn: &SqliteConnection, in_json_login: &InternalMessage) -> Result<InternalResponseMessage, String> {
        let output = InternalResponseMessage::new_value("logout_response", in_json_login.request.msg_id.clone(), 
                                                  Value::Null, 
                                                  0);
        Ok(output)
    }

    pub fn login(conn: &SqliteConnection, in_json_login: &InternalMessage) -> Result<InternalResponseMessage, String> {
        // Decode the JSON object
        let in_user : LoginStruct = match serde_json::from_value(in_json_login.request.parameters.clone()) {
            Ok(u) => u,
            Err(e) => {
                let error_msg : String = format!("ERROR: Decoding JSON Login message: {}", e);
                error!("{}", error_msg);

                return Err( error_msg );
            },
        };

        // Check if the user already exists (in the DB)
        // First by username
        let mut tmp_user = User::by_username(conn, &in_user.username_email);

        if tmp_user.is_none() == true {
            // Second by email
            tmp_user = match User::by_email(conn, &in_user.username_email) {
                Some(u) => Some(u),
                None => return Err(String::from("ERROR: User does not exist")),
            }
        } 
        let read_user : User = tmp_user.unwrap();
        // println!("Read password {:?}", read_user.password.clone());
        // println!("In password {:?}", in_user.password.clone());

        if read_user.password != in_user.password {
            return Err(String::from("ERROR: Invalid password"));
        }

        // Generate JWT Token
        let token = match Claims::create_token(&read_user, TOKEN_DURATION_MINS) {
            Ok(t) => t,
            Err(_e) => {
                return Err(String::from("ERROR: Unable to generate Token"));
            },
        };

        let login_response : LoginResponseStruct = LoginResponseStruct {
            user_id :      read_user.id,
            jwt_token:     token,
        };
       
        let output = InternalResponseMessage::new_value("login_response", in_json_login.request.msg_id.clone(), 
                                                  json!(login_response), 
                                                  0);
        Ok(output)
    }
}
