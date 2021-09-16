/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * It will implement the entry point and the REST API
 */

//#![deny(warnings)]
//#![deny(unused_imports)]

use std::env;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::process;
use std::fs;
use std::result::Result;
use std::sync::{Arc, RwLock};
use futures::executor::block_on;

// Serialize/Deserialize; YAML, JSON
//use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Log 
use log::{debug, error, info, trace, warn};
use log::{LevelFilter, SetLoggerError};
// use log4rs::append::console::{ConsoleAppender, Target};
// use log4rs::append::file::FileAppender;
// use log4rs::config::{Appender, Config, Root};
// use log4rs::encode::pattern::PatternEncoder;
// use log4rs::filter::threshold::ThresholdFilter;

// Date & Time
use chrono::{DateTime, Utc};

// Actix Web Server
use actix_web::{rt::System, web, App, HttpResponse, HttpServer, Responder, HttpRequest, /*middleware*/};
use actix_web::http::StatusCode;
use actix_files as actixfs;
//use actix_files;


// New Nanomsg
use nng::options::{Options, protocol::pubsub::Subscribe};
use nng::{Message, Protocol, Socket, Error};

// Database access and connection pools
// Important: It has to be included in the Root file
#[macro_use]
extern crate diesel;

#[macro_use]
use lazy_static::lazy_static;


// Messages
mod fds_messages;
use fds_messages::*;

mod authorization_manager;
use authorization_manager::check_authorization;

// Manage external modules
mod modules_manager;
use modules_manager::*;

// Manage on-going asynchronous tasks
mod tasks_manager;
use tasks_manager::*;

// Future. Wait for an external task to be completed
mod wait_for_task;
use wait_for_task::*;

// Common functions
mod common;
use common::*;

// Users
mod users;
use users::*;
mod claims;

// SQL Tables definitions and management
mod db;
use db::*;
use db::http_access::*;

// limit the maximum amount of data that server will accept
const MAX_SIZE_JSON : usize =  262_144;

const FDSAAS_VERSION : &str = "0.1";


#[derive(Clone)]
enum EnumStatus {
    NONE,
    RUNNING,
    STOPPED,
}

impl EnumStatus {
    fn to_string(&self) -> String {
        match *self {
            EnumStatus::NONE     => String::from("None"),
            EnumStatus::RUNNING  => String::from("Running"),
            EnumStatus::STOPPED  => String::from("Stopped"),
        }
    }
}


struct GlobalData {
    fds_status :           EnumStatus,
    nng_ip_address :       String,
    nng_socket :           Option<nng::Socket>,
    exit_flag :            bool,
    db_pool:               DbPool,
}

impl GlobalData {
    pub fn new() -> Self 
    {
        GlobalData {
            fds_status :           EnumStatus::NONE,
            nng_ip_address :       String::new(),
            nng_socket :           None,
            exit_flag :            false, 
            db_pool :              establish_connection(),
        }
    }
}


/**
 * Global configuration of the service
 * It also signals when to stop all lops
 */
lazy_static! {
    static ref GLOBAL_DATA : Arc< RwLock< GlobalData >>  = Arc::new( RwLock::new( GlobalData::new() ) );
}





//
// ====================================================================
// ====================================================================
// 



pub fn stop_all_loops() 
{
    // Stop the other loops
    //let mut tmp_data = GLOBAL_DATA.write().unwrap();
    let mut tmp_data = match GLOBAL_DATA.try_write() {
        Ok(d) => d,
        Err(e) => {
            error!("Unable to acquire RwLock write. Error: {}", e);
            return;
        }
    };
    tmp_data.exit_flag = true;
}


/**
 * It does receives messages from the HTTP thread or the NNG thread
 * After processing them, send the reply back to the caller
 * It returns an InternalMessageResponse
 * 
 */
fn main_control_loop() 
{
    info!("**** Starting MAIN control loop ");

    let tmp_config_data = CONFIG_VARIABLES.read().unwrap();

    // Create REP Socket
    // ---------------------------------------------------
    let rep_control_socket = Socket::new(Protocol::Rep0);
    let rep_control_socket   = match rep_control_socket  {
        Ok(s) =>  { info!("Socket to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create main control REP socket. Error: {}", e.to_string());
            stop_all_loops( );
            return;
        },
    };

    // Start listening
    let unused_result = rep_control_socket.listen( tmp_config_data.fds_int_req_address.as_str() );

    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        stop_all_loops( );
        return;
    }
    info!("Correctly connected to Main Control (REP). Address: {}", tmp_config_data.fds_int_req_address);

    
    // Create the Module Manager
    // ----------------------------------------------
    //let mut modules_manager = ModuleManager::new();
    //info!("Module Manager created");

    // Load Module definitions
    // Start executing the modules
    if let Err(_e) = ModuleManager::current().load_module_definitions() {
        // Stop or Kill all running modules
        ModuleManager::current().kill_all_modules();

        stop_all_loops( );
        return;
    } else {
        info!("Module definitions correctly loaded");
    }
    
    let mut done_flag = false;
    
    while done_flag == false {
        {
            let tmp_global_data = GLOBAL_DATA.read().unwrap();
            if tmp_global_data.exit_flag == true {
                done_flag = true;
                debug!("MAIN loop exiting");
                continue;
            }
        }

        // Read a REP message
        // ------------------------------------------
        // This code shall be used when nanomsg socket is being used
        // let input_msg = rep_control_socket.recv();
        // //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        // if let Err(e) = input_msg {
        //     error!("Error when receiving a message: {}", e );
        //     // End of the loop
        //     break;
        // }

        let input_msg = rep_control_socket.try_recv();
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        if let Err(e) = input_msg {
            // If it is not a timeout, process the error
            //if e != Error::TimedOut {
            if e != Error::TryAgain {
                error!("Error when receiving a message: {}", e );
                // End of the loop
                break;
            } else {
                // Small sleep
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        }

        // As u8[]
        let json_buffer = input_msg.unwrap();

        debug!("Received MAIN message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );

        // Decode JSON
        let json_message = serde_json::from_slice(json_buffer.as_slice());
        let mut json_message : RestRequest = match json_message {
            Ok(msg) => {
                msg 
            },
            Err(e) => {
                let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );
        
                let resp_json_message = InternalResponseMessage::new_error("0", tmp_msg.as_str(), 0);
        
                let socket_m = nng::Message::from(resp_json_message.to_string().as_bytes());
        
                // Send answer back
                rep_control_socket.send(socket_m).expect("Unable to send answer message");
                continue;
            },
        };

        // Check minimum set of fields
        if let Err(e) = check_parameters(&json_message) {
            let socket_m = nng::Message::from(e.to_string().as_bytes());
            
            // Send answer back
            rep_control_socket.send(socket_m).expect("Unable to send answer message");
            continue;
        }
        
        // Stop FDS server???
        if json_message.msg_code_id == String::from("exit") {
            // Check magical word
            if json_message.parameters["exit_code"] == String::from("XYZZY") {
                done_flag = true;

                info!("*** Main. Leaving");

                let resp_json_message = InternalResponseMessage::new(json_message.msg_id.clone().as_str(), 
                                                            json!({ "response" : "*** Main. Leaving" }), 0);

                //println!("{:?}", resp_json_message);
                let socket_m = nng::Message::from(resp_json_message.to_string().as_bytes());
                
                // Send answer back
                rep_control_socket.send(socket_m).expect("Unable to send answer message");
                continue;
            }
        }
        
        // Process message
        // It has to reply with an InternalResponseMessage
        let response_message = process_message(&mut json_message);

        // Check if there was an error
        let response_json_message = match response_message {
            Ok(msg) => msg,
            
            Err(e) => {
                let tmp_msg = format!("ERROR: Processing JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );

                let resp_json_message = InternalResponseMessage::new_error(json_message.msg_id.clone().as_str(), 
                                                            tmp_msg.as_str(), 0);
                
                let socket_m = nng::Message::from(resp_json_message.to_string().as_bytes());
                
                // Send answer back
                rep_control_socket.send(socket_m).expect("Unable to send answer message");
                continue;
            },
        };

        // Decode JSON the answer at first level
        // let int_json_message  = serde_json::from_str(response_json_message);
        // let output_int_message : InternalResponseMessage = match int_json_message {
        //     Ok(m) => m,
            
        //     Err(e) => {
        //         let error_msg = format!("ERROR: Unable to decode JSON message: {}", e.to_string());
        //         error!("{}", error_msg);
                
        //         let resp_json_message = build_error_str_json(json_message.msg_id.clone().as_str().unwrap(), 
        //                                                     error_msg.as_str(), 0);
                
        //         let socket_m = nng::Message::from(resp_json_message.as_bytes());
                
        //         // Send answer back
        //         rep_control_socket.send(socket_m).expect("Unable to send answer message");
        //         continue;
        //     },
        // };

        // Extract the execution id
        // let mut output_json_message : String = String::new();

        // if output_int_message.header.error_flag == false {
        //     output_json_message = build_answer_str_json(output_int_message.header.msg_id, 
        //                                                 output_int_message.header.error_msg.as_str());
        // } else {
        //     output_json_message = build_error_str_json(output_int_message.header.msg_id, 
        //                                                output_int_message.header.error_msg.as_str());
        // }

        //let output_json_message = response_json_message.to_string();

        // Send back the answer
        //let socket_m = nng::Message::from(output_json_message.as_bytes());
        let socket_m = nng::Message::from(response_json_message.to_string().as_bytes());

        if let Err(e) = rep_control_socket.send(socket_m) {
            let tmp_msg = format!("ERROR: Sending reply from Main Control. Error: {}", nng::Error::from(e) );
            error!("{}", tmp_msg.as_str() );
        }
    }

    // Stop or Kill all running modules
    ModuleManager::current().kill_all_modules();

    // Stop the other loops
    stop_all_loops( );

    info!("**** Stopping MAIN control loop ");
}

/**
 * It processes a message and return an InternalResponseMessage
 */
fn process_message(in_json_message: &mut RestRequest) -> Result<InternalResponseMessage, String> 
{
    // We check the authentication except for the register and login messages
    //if in_json_message["msg_code_id"] == String::from("register") {
    if in_json_message.msg_code_id != "register" && in_json_message.msg_code_id != "login" && 
       // Module replies with a Status Response when it is ready
       in_json_message.msg_code_id != "get_status_response" {
        // Obtain a connection to the database
        let new_conn;

        {   
            let tmp_data = GLOBAL_DATA.read().unwrap();
            new_conn = tmp_data.db_pool.get().unwrap();
        }

        // Check Authorization
        match check_authorization(&new_conn, &in_json_message.authentication_key) {
            Ok(u) =>  {
                // Add the user id to the message
                in_json_message.parameters["user_id"] = json!(u.id.clone());
            },
            Err(_e) => {
                let tmp_msg = format!("Non authorized. Invalid authentication key: {}", &in_json_message.authentication_key);
                error!("{}", tmp_msg.as_str() );
    
                return Ok( InternalResponseMessage::new_error(in_json_message.msg_id.as_str(), tmp_msg.as_str(), 0) );
            },
        };
    }
    
    // Process the message based on its code
    process_a_message(in_json_message)    
}

/**
 * It processes a message and return an InternalResponseMessage
 */
fn process_a_message(in_json_message: &RestRequest) -> Result<InternalResponseMessage, String> 
{
    debug!("Received message code: {}", in_json_message.msg_code_id.as_str() );

    match in_json_message.msg_code_id.as_str() {
        "test" => {
            println!("This is a test");
        },

        // === API REQUESTS. HTTP MESSAGES ==========================
        
        "register" => {
            info!("Register a new user: ");
            
            // Obtain a connection to the database
            let new_conn;
            
            {   
                let tmp_data = GLOBAL_DATA.read().unwrap();
                new_conn = tmp_data.db_pool.get().unwrap();
            }
            
            return User::register(&new_conn, in_json_message);
        },
        
        "login" => {
            info!("Login a new user: ");

            // Obtain a connection to the database
            let new_conn;
            
            {   
                let tmp_data = GLOBAL_DATA.read().unwrap();
                new_conn = tmp_data.db_pool.get().unwrap();
            }
            return User::login(&new_conn, in_json_message);
        },
        "logout" => {
            info!("logout a new user: ");

            // Obtain a connection to the database
            let new_conn;
            
            {   
                let tmp_data = GLOBAL_DATA.read().unwrap();
                new_conn = tmp_data.db_pool.get().unwrap();
            }
            return User::logout(&new_conn, in_json_message);
        },
                    
        "create_mission" => {
            return ModuleManager::current().call_module(&in_json_message);
        },
        "create_satellite" => {
            return ModuleManager::current().call_module(&in_json_message);
        },
        "create_ground_station" => {
            return ModuleManager::current().call_module(&in_json_message);
        },          

        "orb_propagation" => {
            return ModuleManager::current().call_module(&in_json_message);
        },

        "orb_propagation_tle" => {
            return ModuleManager::current().call_module(&in_json_message);
        },

        "run_script" => {
            return ModuleManager::current().call_module(&in_json_message);
        },



        // === INTERNAL MESSAGES ==========================
        // A module is ready
        "get_status_response" => {
            return ModuleManager::current().module_is_ready(&in_json_message);
        },

        /*
         * Process the answer from the module. Subscribe; Orb Propagation 
         */      
        "orb_propagation_response" => {
            return ModuleManager::current().handle_module_answer(&in_json_message);
        },

        "orb_propagation_tle_response" => {
            return ModuleManager::current().handle_module_answer(&in_json_message);
        },

        "run_script_response" => {
            return ModuleManager::current().handle_module_answer(&in_json_message);
        },

        _ => { println!("Unknown message code: {}", in_json_message.msg_code_id.as_str() );
               error!("Unknown message code: {}. IGNORED", in_json_message.msg_code_id.as_str() );
             }
    };

    Ok( InternalResponseMessage::new(in_json_message.msg_id.as_str(), 
                              json!("Message Ignored"), 0 ) )
}

/**
 * Check the input parameters of the REST
 */
fn check_parameters(in_json_message: &RestRequest) -> Result<bool, InternalResponseMessage> 
{
    // First item to be checked, so, we print the message_id later
    if in_json_message.msg_id.is_empty() == true {
        let tmp_msg = format!("ERROR: Message Id not found. IGNORED");
        error!("{}", tmp_msg.as_str() );

        let resp_json_message = InternalResponseMessage::new_error("0", tmp_msg.as_str(), 0);

        return Err(resp_json_message);
    }

    if in_json_message.version.is_empty() == true {
        let tmp_msg = format!("ERROR: Version not found. IGNORED");
        error!("{}", tmp_msg.as_str() );
        
        let resp_json_message = InternalResponseMessage::new_error(in_json_message.msg_id.as_str(), 
                                                    tmp_msg.as_str(), 0);

        return Err(resp_json_message);
    }

    if in_json_message.version != String::from(REST_JSON_VERSION) {
        let tmp_msg = format!("ERROR: Incorrect version number. IGNORED");
        error!("{}", tmp_msg.as_str() );
        
        let resp_json_message = InternalResponseMessage::new_error(in_json_message.msg_id.as_str(),
                                                     tmp_msg.as_str(), 0);

        return Err(resp_json_message);
    }

    if in_json_message.msg_code_id.is_empty() == true {
        let tmp_msg = format!("ERROR: Msg code not found. IGNORED");
        error!("{}", tmp_msg.as_str() );
        
        let resp_json_message = InternalResponseMessage::new_error(in_json_message.msg_id.as_str(),
                                                     tmp_msg.as_str(), 0);

        return Err(resp_json_message);
    } 

    // authorization_key is not present in Register and Login messages
    if in_json_message.msg_code_id != "login" && in_json_message.msg_code_id != "register" {
        if in_json_message.authentication_key.is_empty() == true {
            let tmp_msg = format!("ERROR: Authentication key not found. IGNORED");
            error!("{}", tmp_msg.as_str() );
    
            let resp_json_message = InternalResponseMessage::new_error_ext(in_json_message.msg_id.as_str(),
                                                             401, tmp_msg.as_str(), 0);
    
            return Err(resp_json_message);
        }
    }
    
    if in_json_message.timestamp.is_null() == true {
        let tmp_msg = format!("ERROR: No Timestamp. IGNORED");
        error!("{}", tmp_msg.as_str() );

        let resp_json_message = InternalResponseMessage::new_error(in_json_message.msg_id.as_str(), 
                                                     tmp_msg.as_str(), 0);

        return Err(resp_json_message);
    }

    return Ok(false);
}

/**
 * It does receives messages via a New Nanomsg socket.
 * It opens 3 sockets; 
 * 1 of BUS type; for generic events 
 * 1 of SUB type for receiving modules responses,
 * 1 of REQ type
 * After that it will forward the messages to the central processing loop.
 * If the socket is REQ, it will send back the reply to the requester
 * 
 */
fn nng_control_loop() 
{
    info!("**** Starting NNG control loop ");

    let tmp_config_data = CONFIG_VARIABLES.read().unwrap();

    // Create BUS Socket
    // --------------------------------------
    let bus_control_socket = Socket::new(Protocol::Bus0);
    let bus_control_socket = match bus_control_socket  {
        Ok(s) =>  { info!("NNG Socket BUS to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("NNG Unable to create FDS BUS socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = bus_control_socket.listen( tmp_config_data.fds_nng_bus_address.as_str() );
    if let Err(e) = unused_result {
        error!("NNG Error when starting listening the socket: {}", e );
        return;
    }
    info!("NNG Correctly connected to FDS BUS. Address: {}", tmp_config_data.fds_nng_bus_address);

    // Set socket timeout
    //bus_control_socket.set_opt::<nng::options::RecvTimeout>( Some(Duration::from_millis(100)) ).unwrap(); 

   

    // Create REP Socket
    // --------------------------------------
    let rep_control_socket = Socket::new(Protocol::Rep0);
    let rep_control_socket = match rep_control_socket  {
        Ok(s) =>  { info!("NNG Socket REP to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("NNG Unable to create FDS REP socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = rep_control_socket.listen( tmp_config_data.fds_nng_rep_address.as_str() );
    if let Err(e) = unused_result {
        error!("NNG Error when starting listening the FDS REP socket: {}", e );
        return;
    }
    info!("Correctly connected to FDS REP. Address: {}", tmp_config_data.fds_nng_rep_address);


    // Create SUB Socket
    // ---------------------------------------------------
    let sub_control_socket = Socket::new(Protocol::Sub0);
    let sub_control_socket = match sub_control_socket  {
        Ok(s) =>  { info!("NNG Socket SUB to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("NNG Unable to create FDS SUB socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start dialing
    let unused_result = sub_control_socket.listen( tmp_config_data.fds_nng_sub_address.as_str() );
    if let Err(e) = unused_result {
        error!("NNG Error when starting dialing the socket: {}", e );
        return;
    }
    info!("NNG Correctly connected to FDS SUB Address: {}", tmp_config_data.fds_nng_sub_address);

    // Set socket timeout
    //sub_control_socket.set_opt::<nng::options::RecvTimeout>( Some(Duration::from_millis(100)) ).unwrap(); 
    sub_control_socket.set_opt::<Subscribe>( vec![] ).unwrap();


    // Main control loop REQ socket
    let main_control_socket = Socket::new(Protocol::Req0);
    let main_control_socket = match main_control_socket  {
        Ok(s) =>  { 
            info!("NNG Socket to Main Control correctly created ");
            s
        },
        Err(e) => {
            error!("NNG Unable to create FDS REQ socket. Error: {}", e.to_string());
            return;
        },
    };

    // Get the address of main control loop
    let controller_address;
    {
        let tmp_global_data = GLOBAL_DATA.read().unwrap();
        
        controller_address = tmp_global_data.nng_ip_address.clone();
    }
    debug!("Controller address: {}", controller_address);

    // Start listening
    let _unused_result = main_control_socket.dial( controller_address.as_str() );
    if let Err(e) = _unused_result {
        error!("NNG Error when starting dialing the Main Control server: {}", e );
        return;
    }
    info!("NNG Correctly connected to Main Control server. Address: {}", controller_address);

    let mut done_flag = false;
    
    while done_flag == false {
        {
            let tmp_global_data = GLOBAL_DATA.read().unwrap();
            if tmp_global_data.exit_flag == true {
                done_flag = true;
                debug!("NNG loop exiting");
                continue;
            }
        }

        

        // Read a message
        // ------------------------------------------
        // This code shall be used when nanomsg socket is being used
        //let input_msg = bus_control_socket.recv();

        let input_msg = bus_control_socket.try_recv();
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        if let Err(e) = input_msg {
            // If it is not a timeout, process the error
            if e != Error::TryAgain {
                error!("Error when receiving a BUS message: {}", e );
                // End of the loop
                break;
            }
        } else {
            // As u8[]
            let json_buffer = input_msg.unwrap();
    
            debug!("Received DBUS message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );
    
            // Forward the message to the main control loop as a REQ/REP
            let payload = String::from_utf8( json_buffer.as_slice().to_vec()).unwrap();
            block_on( forward_message_internal(&main_control_socket, &payload) ).unwrap();

            // Ignore the answer
        }


        // Read SUB message
        // ------------------------------------------
        // This code shall be used when nanomsg socket is being used
        let input_msg = sub_control_socket.try_recv();
        
        // match input_msg {
        //     Ok(m) => {
        //         // As u8[]
        //         debug!("Received SUB message: {}", String::from_utf8( m.as_slice().to_vec()).unwrap() );

        //         // Forward the message to the main control loop as a REQ/REP
        //         let payload = String::from_utf8( m.as_slice().to_vec()).unwrap();
        //         block_on( forward_message_internal(&main_control_socket, &payload) ).unwrap();

        //         // Ignore the answer
        //     },
        //     Err(e) => {
        //         // If it is not a timeout, process the error
        //         if e != Error::TryAgain {
        //             error!("Error when receiving a SUB message: {}", e );
        //             // End of the loop
        //             break;
        //         }
        //     },
        // };

        if let Err(e) = input_msg {
            // If it is not a timeout, process the error
            if e != Error::TryAgain {
                error!("Error when receiving a SUB message: {}", e );
                // End of the loop
                break;
            } 
        } else {
            // As u8[]
            let json_buffer = input_msg.unwrap();
    
            debug!("Received SUB message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );
    
            // Forward the message to the main control loop as a REQ/REP
            let payload = String::from_utf8( json_buffer.as_slice().to_vec()).unwrap();
            block_on( forward_message_internal(&main_control_socket, &payload) ).unwrap();

            // Ignore the answer
        }



        // Read a message
        // ------------------------------------------
        // This code shall be used when nanomsg socket is being used
        let input_msg = rep_control_socket.try_recv();
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        if let Err(e) = input_msg {
            // If it is not a timeout, process the error
            if e != Error::TryAgain {
                error!("Error when receiving a REQ message: {}", e );
                // End of the loop
                break;
            }
        } else {
            // As u8[]
            let json_buffer = input_msg.unwrap();
    
            debug!("Received REQ message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );
    
            // Forward the message to the main control loop as a REQ/REP
            let payload = String::from_utf8( json_buffer.as_slice().to_vec()).unwrap();

            // Send the answer back
            let output_msg = block_on( forward_message_internal(&main_control_socket, &payload) );
            let output_msg : String = match output_msg {
                Ok(o) => o,
                Err(e) => {
                    let error_msg = format!("Error when forwarding JSON message. Error: {}", e);
                    error!("{}", error_msg);

                    error_msg
                },
            };
            
            let tmp_message = Message::from(output_msg.as_bytes());

            // Send the message to the main control loop
            // Message is a JSON
            debug!("Sending REP message: {}", output_msg );

            let status = rep_control_socket.send(tmp_message);
            if let Err(e) = status {
                error!("Error sending reply back to the Requester: {:?}. IGNORED", e );
            }
        }
        
        // Small sleep
        thread::sleep(Duration::from_millis(100));
    }

    info!("**** Stopping NNG control loop ");
}


// Main web page
/**
 * Read the WebContent index.html page and return it
 */
// async fn index() -> impl Responder {


// let path: PathBuf = req.match_info().query("filename").parse().unwrap();
// Ok(NamedFile::open(path)?)

// //let f = fs::Files::new("/", "WebContent").index_file("index.html");
// //return HttpResponse::Ok().body(   );
// return HttpResponse::Ok().body("FDS as a Service\n\nIncomplete Worlds (c) 2020");
// }





/**
 * Return the status of the FDS module
 * Status can be; None, Running, Stopped
 */
async fn get_status() -> impl Responder 
{
    info!("   *** Get Status");

    let tmp_data = GLOBAL_DATA.read().unwrap();
    // let tmp_status = tmp_data.fds_status.to_string();

    let tmp_status = GetStatusResponseStruct{ status : tmp_data.fds_status.to_string() };

    return HttpResponse::Ok().json( tmp_status );
}

/**
 * Return the current version of the API
 */
async fn get_version() -> impl Responder 
{
    info!("   *** Get Version");

    // Read-only. So, it should be fine
    let tmp_version = GetVersionResponseStruct{ version : FDSAAS_VERSION.to_string() };

    return HttpResponse::Ok().json( tmp_version );
}

/**
 * Return the descrption of the selected operation.actix_files
 * It describes how to use the operation
 */
async fn api_usage(in_operation : web::Path<String>) -> impl Responder 
{
    info!("   *** Get API Usage");

    let usage_msg : String;

    match in_operation.as_str() {
        "register_message" => {
            usage_msg = fs::read_to_string("doc/register.html").expect("Unable to read 'doc/register.html' file");
        },

        "create_mission" => {
            usage_msg = fs::read_to_string("doc/create_mission.html").expect("Unable to read 'doc/create_mission.html' file");
        },

        "create_satellite" => {
            usage_msg = fs::read_to_string("doc/create_satellite.html").expect("Unable to read 'doc/create_satellite.html' file");
        },

        "create_ground_station" => {
            usage_msg = fs::read_to_string("doc/create_ground_station.html").expect("Unable to read 'doc/create_ground_station.html' file");
        },

        "orb_propagation" => {
            usage_msg = fs::read_to_string("doc/orb_propagation.html").expect("Unable to read 'doc/orb_propagation.html' file");
        },

        "orb_propagation_tle" => {
            usage_msg = fs::read_to_string("doc/orb_propagation_tle.html").expect("Unable to read 'doc/orb_propagation_tle.html' file");
        },

        "run_script" => {
            usage_msg = fs::read_to_string("doc/run_script.html").expect("Unable to read 'doc/run_script.html' file");
        },

        _ => { 
            usage_msg = format!("Unknown operation name: {}", in_operation);
        }
    };

    return HttpResponse::Ok()
                        .content_type("text/html")
                        .body( usage_msg );
}


/**
 * Forward a message to the Main Control loop and wait for the answer
 * Received message shall be of type InternalMessage
 * 
 */
async fn forward_and_wait(in_socket: &Socket, in_payload: &String) -> Result<String, String> 
{
    let internal_message : String = match forward_message_internal(in_socket, in_payload).await {
        Ok(m) => m,
        Err(e) => {
            return Err(e);
        },
    };
    
    // Extract the execution id
    // Decode JSON at first level
    let json_message1 = serde_json::from_str( internal_message.as_str() );
    let json_message : InternalResponseMessage = match json_message1 {
        Ok(msg) => msg,
        Err(e) => {
            let error_msg = format!("ERROR: Unable to decode Internal Response JSON message: {}", e.to_string());
            error!("{}", error_msg);

            //return Err( String::from(error_msg) );
            return Err( RestResponse::new_error_msg(&String::from("-1"), error_msg).to_string() );
        },
    };

    // If there was an error, return the Response (ErrorData)
    let tmp_error = json_message.error.clone();
    match tmp_error {
        Some(e) => {
            // let http_output = ErrorStruct::new(e);
            // return Err( http_output.to_string() );
            return Err( RestResponse::new(&json_message).to_string() );
        },
        None => {},
    };

    // Obtain the answer to the HTTP Request
    let http_output : String;

    if json_message.wait_flag == true {
        // Wait for answer
        let new_wait_task = WaitForAnswerFuture::new(json_message.execution_id);
    
        new_wait_task.await;
    
        // Get answer
        let tmp_http_output;
        {
            tmp_http_output = TASK_MANAGER.read().unwrap().get_answer(json_message.execution_id);
        }
        
        http_output = match tmp_http_output {
            Ok(m) => //m,
            {
                RestResponse::new_value(&json_message.msg_id, json!(m)).to_string()
            },
            Err(e) => {
                return Err(e);
            },
        };
    } else {
        // We do not have to wait, return the current response
        // http_output = json_message.parameters.to_string();
        http_output = RestResponse::new(&json_message).to_string();
    }

    debug!("REPLYING with received message: {}", http_output);

    return Ok( http_output );
}

/**
 * Forward the message to the Main Control loop and return the answer
 * Answer can be a String containing any type of JSON object
 */
async fn forward_message_internal(in_socket: &Socket, in_payload: &String) -> Result<String, String> 
{
    debug!("INT FORWARD sending message: {}", in_payload);

    // Send the message to main control loop
    let tmp_message = Message::from(in_payload.as_bytes());

    let status = in_socket.send(tmp_message);
    if let Err(e) = status {
        let error_msg = format!("Error sending HTTP request payload to the Main Control: {:?}", e );
        error!("{}", error_msg);
        return Err( error_msg );
    }

    // Receive the answer and process it
    // Message is a JSON
    let recv_message = in_socket.recv();
    let recv_message = match recv_message {
        Ok(m) => m,
        Err(e) => {
            let error_msg = format!("Unable to receive message from main control loop. Error: {}", e.to_string());
            error!("{}", error_msg);
            return Err( error_msg );
        },
    };

    let received_data = recv_message.as_slice();
    let output_buffer = String::from_utf8( received_data.to_vec() ).unwrap();
    debug!("Received answer: {}", output_buffer);

    Ok( output_buffer )
}

/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn forward_message(in_payload: String, in_request: HttpRequest, in_db: web::Data<DbPool>) -> impl Responder 
{
    debug!("HTTP FORWARD sending message: {}", in_payload );

    // Record the HTTP access
    record_access(&in_request, &in_db);

    // Create a socket with the Main Control loop
    let tmp_control_socket;

    {
        let tmp_data = GLOBAL_DATA.read().unwrap();
        tmp_control_socket = tmp_data.nng_socket.clone();
    }

    // Check if the socket already exists, if not create it
    let tmp_control_socket = match tmp_control_socket {
        Some(s) => {
            debug!("Socket to main already exist. Nothing to be done");
            s
            },
        None => { 
            // Create Socket, if it does not exist
            let new_control_socket = Socket::new(Protocol::Req0);
            let new_control_socket = match new_control_socket  {
                Ok(s) =>  { 
                    info!("Socket to Main Control correctly created ");
                    s
                },
                Err(e) => {
                    error!("Unable to create Main Control REP socket. Error: {}", e.to_string());
                    return HttpResponse::InternalServerError().body( e.to_string() );
                },
            };

            let controller_address;
            {
                let tmp_data = GLOBAL_DATA.read().unwrap();
                controller_address = tmp_data.nng_ip_address.clone();
            }

            // Start transmitting
            let unused_result = new_control_socket.dial( controller_address.as_str() );
            if let Err(e) = unused_result {
                error!("Error when starting listening the socket: {}", e );
                return  HttpResponse::InternalServerError().body( e.to_string() );
            }
            info!("Correctly connected to Main Control server. Address: {}", controller_address);

            // Store the socket
            {
                let mut tmp_data = GLOBAL_DATA.write().unwrap();

                // let mut tmp_data = match GLOBAL_DATA.try_write() {
                //     Ok(d) => d,
                //     Err(e) => {
                //         error!("Unable to acquire RwLock.write. Error: {}", e);
                //         return HttpResponse::InternalServerError().body( e.to_string() );
                //     }
                // };
                
                tmp_data.nng_socket = Some( new_control_socket.clone() );
            }
            
            new_control_socket
        },
    };

    debug!("Socket to main created");

    // Send the message to the main control loop and wait asynchronously for the answer
    // Then reply to the HTTP handler
    let http_output = forward_and_wait(&tmp_control_socket, &in_payload).await;

    match http_output {
        Ok(o) => {
            return HttpResponse::Ok()
                        .content_type("application/json")
                        .body( o );
        },
        Err(e) => {
            return HttpResponse::InternalServerError().body( e );
        },
    };
}

/**
 * Return index.html
 */
// async fn index(req: HttpRequest) -> actix_web::Result<fs::NamedFile> {
async fn index(in_request: HttpRequest, in_db: web::Data<DbPool>) -> actix_web::Result<actixfs::NamedFile>  { // -> impl Responder
    debug!("   *** Index");

    // Record the HTTP access
    record_access(&in_request, &in_db);

    Ok( actixfs::NamedFile::open("./WebContent/index.html")?.set_status_code(StatusCode::NOT_FOUND) )

//     //let f = fs::Files::new("/", "WebContent").index_file("index.html");

//     //let path: PathBuf = req.match_info().query("filename").parse().unwrap();
//     //Ok( NamedFile::open( path )? )

//     // response
//     // Ok(HttpResponse::build( StatusCode::OK)
//     //     .content_type("text/html; charset=utf-8")
//     //     .body( HttpResponse::include_str!("../static/welcome.html")) )


//     // return HttpResponse::build( StatusCode::OK)
//     //     .content_type("text/html; charset=utf-8")
//        // .body( include_str!("../static/welcome.html"));
//     Ok ( fs::NamedFile::open("../WebContext/index.html")? )
}

/**
 * Not Allowed Method handler
 * Record the access in the database
 */
async fn not_allowed_method(in_payload: String, in_request: HttpRequest, in_db: web::Data<DbPool>) -> impl Responder 
{
    debug!("Not Allowed Method: {}", in_payload );

    // Record the HTTP access
    record_access(&in_request, &in_db);

    HttpResponse::MethodNotAllowed()
}

fn record_access(in_request: &HttpRequest, in_db: &web::Data<DbPool>)
{
    // Record the HTTP access
    let tmp_conn = in_db.get().unwrap();
    let tmp_address : String;
    let tmp_hostname : String;

    match in_request.connection_info().realip_remote_addr() {
        Some(r) => tmp_address = String::from(r),
        None    => tmp_address = String::from("None"),
    };

    tmp_hostname = String::from(in_request.connection_info().host());

    if let Err(e) = HttpAccess::create(&tmp_conn, 
                                        chrono::Local::now().naive_local(),
                                        &tmp_address,
                                        &tmp_hostname,
                                        &String::from(in_request.path()) ) {
        warn!("Error creating HTTP Access record: {}. IGNORED", e );
    } 
}

fn usage() 
{
    println!("Incomplete Worlds (c) 2020");
    println!("Flight Dynamics System (FDS) as a Service - HTTP API");
    println!("");
    println!("   Usage:    main   config_file_name");
    println!("");
}



// ================================================================
// *
// *  M  A  I  N
// *
// ================================================================
#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 2 {
        usage();
        return Ok(());
    }
    
    // Write the PID to a file
    let data = format!("{}", process::id());
    fs::write("fdsaas.pid", data).expect("Unable to write 'fdsaas.pid' file");

    let mut now: DateTime<Utc> = Utc::now(); 

    println!("**********************************");
    println!("Initializing FDS as a Service - HTTP API : {}", now.naive_utc() );
    println!("**********************************");

    // Load configuration file
    debug!("Reading the configuration file");
                
    let tmp_config_file_name = args[1].clone();
    let config_variables = read_config_json(&tmp_config_file_name);
    let config_variables = match config_variables {
        // Just return the variables
        Ok(tmp_variables) => tmp_variables,
        Err(tmp_error) => {
            println!("Unable to read the configuration file: {}", tmp_config_file_name.as_str() );
            println!("Error: {}", tmp_error);
            return Ok(());
        }
    };

    // Init Log
    let tmp_log_filename = config_variables.config_log_filename.clone();

    let error_code = config_log(&tmp_log_filename);
    if let Err(e) = error_code {
        println!("ERROR: Unable to read the Log Configuration file: {}", tmp_log_filename.as_str());
        println!("{} {}", e.to_string(), e);
        return Ok(());
    }

    info!("**********************************");
    info!("Initializing FDS as a Service - HTTP API : {}", now.naive_utc() );
    info!("**********************************");

    // External HTTP Address. It will listen for HTTP requests coming from this address
    let http_address = config_variables.fdsaas_http_address.clone();
   
    info!("Listening HTTP IP Address: {}", http_address);

    
    // Connect to the FDS server via Nanomsg 
    info!("Nanomsg Internal Address: {}", config_variables.fds_int_req_address);

    // Creatint database connection pool
    let conn_pool = db::establish_connection();
    info!("Connection pool to database created");

    // Copy to be transfered to HTTP server
    let conn_pool_copy = conn_pool.clone();

    // Data shared between all threads
    {
        let mut tmp_data = GLOBAL_DATA.write().unwrap();

        tmp_data.fds_status      = EnumStatus::RUNNING;
        tmp_data.nng_ip_address  = config_variables.fds_int_req_address.clone();
        tmp_data.db_pool         = conn_pool;
    }

    // Store in the shared global variable
    {
        let mut tmp_config_data = CONFIG_VARIABLES.write().unwrap();

        *tmp_config_data = config_variables;
    }

    
    // Channel for retrieving the http server variable
    let (tx, rx) = mpsc::channel();
    
    // Start http server in a separated thread
    let _http_thread = thread::spawn(move || {
        let sys = System::new("http-server");

        let srv = HttpServer::new(move || {
            App::new()
    
            // limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit( MAX_SIZE_JSON ))
    
            // Pass data to the handler. It makes a copy
            .data(  conn_pool_copy.clone() )
            //.data( http_global_data1.clone())
            //.app_data( http_global_data1.clone() )

            .service(
                web::scope("/fdsaas")
                    .default_service(
                        web::route().to(not_allowed_method),
                    )

                    // .route("/", web::get().to( index ) )
                    // .route("/index.html", web::get().to( index ) )
                    .route("/api/exit", web::get().to(forward_message))

                    // USERS MANAGEMENT
                    // ---------------------------------
                    .route("/api/register", web::put().to(forward_message))
                    .route("/api/login", web::get().to(forward_message))
                    .route("/api/logout", web::get().to(forward_message))
                    
                    // GENERAL
                    // ---------------------------------
                    .route("/api/{operation}/usage", web::get().to(api_usage))
                    .route("/api/version", web::get().to(get_version))
                    .route("/api/status", web::get().to(get_status))

                    // TOOLS
                    // ---------------------------------
                    .route("/api/create_mission", web::post().to(forward_message))
                    .route("/api/create_satellite", web::post().to(forward_message))
                    .route("/api/create_ground_station", web::post().to(forward_message))

                    // PROPAGATE AN ORBIT
                    // ---------------------------------
                    .route("/api/orb_propagation", web::get().to(forward_message))
                    .route("/api/orb_propagation_tle", web::get().to(forward_message))

                    // Execute a plain GMAT script
                    // ---------------------------------
                    .route("/api/run_script", web::get().to(forward_message))

                    // /api/list - List all APIs
            )
            
            // Root URL
            // work, but serves only index.html
            //.service( actixfs::Files::new("/", "./WebContent/").index_file("index.html") )
            .service(
                web::scope("/")
                    .default_service(
                        web::route().to(not_allowed_method),
                    )
                .route("/index.html", web::get().to(index))
            )
            

        })
        .bind(http_address)?
        .run();

        let _ = tx.send( srv );
        let sys_result = sys.run();
        
        info!("Ctrl-C received, shutting down");
        stop_all_loops();

        sys_result
    });

    let srv = rx.recv().unwrap();
    
    // NNG (BUS, REQ, SUB) control loop
    let nng_thread = thread::spawn(move || {
        nng_control_loop();
    });

    // Main control loop thread
    main_control_loop();

    nng_thread.join().unwrap();
    
    // Small sleep
    thread::sleep(Duration::from_secs(1));

    // pause accepting new connections
    //srv.pause().await;
    // resume accepting new connections
    //srv.resume().await;
    // Gratecul Stop of server
    srv.stop(true).await;

    // Remove the pid file
    fs::remove_file("fdsaas.pid").expect("Unable to remove 'fdsaas.pid' file");

    now = Utc::now();
    info!("**********************************");
    info!("Finishing FDS Server: {}", now.naive_utc());   
    info!("**********************************");

    Ok(())
}
