/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * Functions to manage Users; CRUD
 */

// JSON serialization
use serde::{Deserialize, Serialize};
use serde_json::{Value};

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


use crate::schema::*;
use crate::fds_messages::*;




// Note: u32 cannot be used. It has to be i32

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_user"]
pub struct User {
    pub id:                   String,
    pub username:             String,
    // Hashed password. So, it is not stored in clear
    pub password:             String,
    pub email:                String,
    pub authentication_key:   String,
    pub created:              chrono::NaiveDateTime,
} 

impl User 
{
    /**
     * Register a new user.
     * The JSON message shall include the user name, hashed password and the email
     * Document: doc/register.txt
     * {
        "msg_code_id" :           "register",
        "authentication_key" :    "",
        "user_id" :               "",

        "username" :              "john_doe",
        "password" :              "9Sec$reZATpwd0$",
        "email" :                 "john_doe@someaddress.com",
        }
     */
    pub fn register(conn: &SqliteConnection, in_json_message: &Value) -> Result<String, String> 
    {
        // Decode JSON
        let json_message = serde_json::from_value( in_json_message.clone() );
        let register_message : RegisterMessage = match json_message {
                Ok(msg) => msg,  
                Err(e) => {
                    let tmp_msg = format!("ERROR: Unable to decode JSON RegisterMessage: {}", e.to_string());
                    error!("{}", tmp_msg.as_str() );
                    return   Err(tmp_msg);
                },
        };

        // if email.is_none() && phone.is_none() {
        //     return None
        // } 

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


        /*
        let new_username : String = match in_json_message["username"].as_str() {
            Some(n) => n.into(),
            None => {
                let error_msg = String::from("username field not found in JSON object");
                error!("{}", error_msg);
                return Err(error_msg);
            },
        };

        // TODO: In Hash
        let new_password : String = match in_json_message["password"].as_str() {
            Some(n) => n.into(),
            None => {
                let error_msg = String::from("password field not found in JSON object");
                error!("{}", error_msg);
                return Err(error_msg);
            },
        };

        let new_email : String = match in_json_message["email"].as_str() {
            Some(n) => n.into(),
            None => {
                let error_msg = String::from("email field not found in JSON object");
                error!("{}", error_msg);
                return Err(error_msg);
            },
        };
        */

        // Create and insert the new user into the database
        match User::create(conn, &register_message.username, &new_password, &register_message.email) {
            Ok(nu) => {
                let info_msg : String = format!("Created a new user with uuid: {}", nu.id);
                info!("{}", info_msg);

                let tmp_user = RegisterMessageResponse { 
                    user_id :             nu.id,
                    authentication_key :  nu.authentication_key,
                };

                match serde_json::to_string(&tmp_user) {
                    Ok(o) => return Ok( build_int_answer_str_json( o.as_str() ) ),
                    Err(e) => {
                        let error_msg : String = format!("Error encoding JSON Register message answer: {}", e);
                        error!("{}", error_msg);

                        return Err( build_int_error_str_json(error_msg.as_str() ) );
                    }, 
                };
            },
            Err(e) => {
                let error_msg : String = format!("Error creating user: {}", e);
                error!("{}", error_msg);

                return Err(  build_int_error_str_json(error_msg.as_str() ) );
            },
        };
    }

    /**
     * Insert a new User record in the table
     */
    pub fn create(conn: &SqliteConnection, in_username: &String, in_password: &String,
                    in_email: &String)  -> Result<User, diesel::result::Error>
    {            
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();
        
        // Create a new record
        let new_user = User {
            // TODO: Apply JWT
            authentication_key:   new_uuid.clone(),

            id:                   new_uuid,
            username:             in_username.clone(),
            password:             in_password.clone(),
            email:                in_email.clone(),
            created:              chrono::Local::now().naive_local(),
        };

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

    /**
     * Create a new user struct
     */
    // fn new_user_struct(in_username: &str, in_password: &str, in_email: &str) -> Self 
    // {
    //     let new_uuid = format!("{}", uuid::Uuid::new_v4());
    //     User {
    //         id:         new_uuid,
    //         username:   in_username.into(),
    //         email:      in_email.into(),
    //         password:   in_password.into(),
    //         // TODO
    //         authentication_key:  String::from(""),
    //         created:    chrono::Local::now().naive_local(),
    //     }
    // }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //user_dsl.load::<User>(conn).expect("Error loading users")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: &String) -> Option<Self> 
    {
        if let Ok(record) = t_user::table.find(in_id).first(conn) {
            Some(record)
        } else {
            None
        }
    }

    /*
    pub fn by_email(email_str: &str, conn: &SqliteConnection) -> Option<Self> 
    {
        // use super::schema::users::dsl::email;

        // if let Ok(record) = user_dsl.filter(email.eq(email_str)).first::<User>(conn) {
        //     Some(record)
        // } else {
        //     None
        // }
    }

    pub fn create(email: Option<&str>, phone: Option<&str>, conn: &SqliteConnection) -> Option<Self> 
    {
        // let new_id = Uuid::new_v4().to_hyphenated().to_string();
        
        // if email.is_none() && phone.is_none() {
        //     return None
        // } 
                
        // if phone.is_some() {
        //     if let Some(user) = Self::by_phone(&phone.unwrap(), conn) {
        //         return Some(user)
        //     } 
        // }
        
        // if email.is_some() {
        //     if let Some(user) = Self::by_email(&email.unwrap(), conn) {
        //         return Some(user)
        //     } 
        // }

        // let new_user = Self::new_user_struct(&new_id, phone, email);

        // diesel::insert_into(user_dsl)
        //     .values(&new_user)
        //     .execute(conn)
        //     .expect("Error saving new user");

        // Self::by_id(&new_id, conn)
    }
    */

    
    
}
