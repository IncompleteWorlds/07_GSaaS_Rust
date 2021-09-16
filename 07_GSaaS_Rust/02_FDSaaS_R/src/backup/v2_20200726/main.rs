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
// use std::sync::mpsc::channel;
// use std::collections::HashMap;
//use std::error::Error;
//use std::collections::BTreeMap;
use std::result::Result;
use futures::executor::block_on;
// use std::rc::Rc;
// use std::cell::RefCell;
use std::sync::{Arc, Mutex, RwLock};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{Value};

// Log 
use log::{debug, error, info, trace, warn};
use log::{LevelFilter, SetLoggerError};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

// Date & Time
use chrono::{DateTime, Utc};

// Actix Web Server
use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest /*middleware*/};
use actix_files as fs;
//use actix_files::{ NamedFile };
// use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_rt::System;
use tokio::time::delay_for;

// New Nanomsg
use nng::options::{Options};
use nng::{Aio, AioResult, Context, Message, Protocol, Socket, Error};

#[macro_use]
use lazy_static::lazy_static;

//#[macro_use]
//extern crate lazy_static;


// Messages
mod fds_messages;
use fds_messages::*;
//use fds_messages::{ApiMessage, build_api_message, build_api_message_str, build_api_answer, build_api_answer_str, build_api_answer_str_json};

mod authorization_manager;
use authorization_manager::check_authorization;

mod modules_manager;
use modules_manager::*;

mod tasks_manager;
use tasks_manager::*;

mod wait_for_task;
use wait_for_task::*;

mod process_message_task;
use process_message_task::ProcessMessageFuture;

// Common functions
mod common;
//use common::{ read_config_json, config_log, ConfigVariables };
use common::*;
use common::CONFIG_VARIABLES;




// limit the maximum amount of data that server will accept
const MAX_SIZE_JSON : usize =  262_144;

const FDSAAS_VERSION : &str = "0.1";

// NNG number of parallel workers
const PARALLEL: usize = 128;

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

#[derive(Serialize, Deserialize)]
struct InternalMessage {
    execution_id:   u32,
    payload:        String,
}

struct GlobalData {
    fds_status :           EnumStatus,
    nng_ip_address :       String,
    nng_socket :           Option<nng::Socket>,
    exit_flag :            bool,
}

impl GlobalData {
    pub fn new() -> Self 
    {
        GlobalData {
            fds_status :           EnumStatus::NONE,
            nng_ip_address :       String::new(),
            nng_socket :           None,
            exit_flag :            false, 
        }
    }
}

/**
 * Global configuration of the service
 * It also signals when to stop all lops
 */
lazy_static! {
    static ref GLOBAL_DATA : Arc< RwLock< GlobalData >>  = Arc::new( RwLock::new( GlobalData::new() ) );

//    static ref CONFIG_VARIABLES : Arc< RwLock< ConfigVariables >>  = Arc::new( RwLock::new( ConfigVariables::new() ) );
}







//
// ====================================================================
// ====================================================================
// 

fn stop_all_loops() 
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
 * 
 */
async fn main_control_loop() 
{
    info!("**** Starting MAIN control loop ");

    let tmp_config_data = CONFIG_VARIABLES.read().unwrap();

    // Create PULL Socket
    // ---------------------------------------------------
    let rep_control_socket = Socket::new(Protocol::Pull0);
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
    if let Err(_e) = ModuleManager::current().load_module_definitions().await {
        // Stop or Kill all running modules
        ModuleManager::current().kill_all_modules().await;

        stop_all_loops( );
        return;
    } else {
        info!("Module definitions correctly loaded");
    }

    // Get a reference to the Task Manager
    // ----------------------------------------------
    //let task_manager : &mut TaskListManager;
    // let task_manager : &mut TaskListManager;
    // {
    //     let global_data = in_global_data.lock().unwrap();
    //     &mut match global_data.tasks_manager {
    //         Some(tm) => tm,
    //         None => {
    //             error!("TaskListManager is not defined");
    //             // Stop or Kill all running modules
    //             modules_manager.kill_all_modules().await;

    //             stop_all_loops();
    //             return;
    //         },
    //     };
    // }
    //let tasks_manager = TaskListManager::current();
    
    let mut done_flag = false;
    let mut output_json_message : String = String::new();
    
    while done_flag == false {
        // unsafe {
            //     if GLOBAL_DATA.exit_flag == true {
                //         done_flag = true;
                //         debug!("MAIN loop exiting");
                //         continue;
                //     }
                // }
        {
            let tmp_global_data = GLOBAL_DATA.read().unwrap();
            if tmp_global_data.exit_flag == true {
                done_flag = true;
                debug!("MAIN loop exiting");
                continue;
            }
        }

        // A message will contain:
        //   Internal message:
        //      execution_id:   u32,
        //      payload:        String,
        //

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
                //thread::sleep(Duration::from_millis(50));
                tokio::time::delay_for(Duration::from_millis(50)).await;
                continue;
            }
        }

        // As u8[]
        let json_buffer = input_msg.unwrap();

        debug!("Received MAIN message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );

        // Decode JSON at first level
        let json_message1 = serde_json::from_slice(json_buffer.as_slice());
        let json_message1 : InternalMessage = match json_message1 {
            Ok(msg) => msg,
            Err(e) => {
                let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );                
                continue;
            },
        };

        // Decode JSON
        let json_message2 = serde_json:: from_str(json_message1.payload.as_str());
        let json_message2 : Value = match json_message2 {
            Ok(msg) => msg,
            Err(e) => {
                let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );
                let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

                // Set the answer to the associated task and flag it as completed
                TaskListManager::current().set_answer_completed(json_message1.execution_id, resp_json_message);
                continue;
            },
        };

        // Check minimum set of fields
        if json_message2["msg_code_id"].is_null() == true {
            let tmp_msg = format!("ERROR: No msg code found. IGNORED");
            error!("{}", tmp_msg.as_str() );
            
            let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

            // Set the answer to the associated task and flag it as completed
            TaskListManager::current().set_answer_completed(json_message1.execution_id, resp_json_message);            
            continue;
        } 

        if json_message2["authentication_key"].is_null() == true {
            let tmp_msg = format!("ERROR: No Authentication key. IGNORED");
            error!("{}", tmp_msg.as_str() );

            let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

            // Set the answer to the associated task and flag it as completed
            TaskListManager::current().set_answer_completed(json_message1.execution_id, resp_json_message);            
            continue;
        }

        // Stop FDS server???
        if json_message2["msg_code_id"] == String::from("exit") {
            // Check magical word
            if json_message2["exit_code"] == String::from("XYZZY") {
                done_flag = true;

                info!("*** Main. Leaving");

                // Send back the answer
                output_json_message = build_api_answer_str_json(false, "", "");

                // Set the answer to the associated task and flag it as completed
                TaskListManager::current().set_answer_completed(json_message1.execution_id, output_json_message);                            
                continue;
            }
        }
        
        // Process message
        let response_message = process_message(&json_message2).await;

        output_json_message = match response_message {
            Ok(msg) => {
                let resp_json_message = build_api_answer_str_json(false, msg.as_str(), "");

                resp_json_message
            },

            Err(e) => {
                let tmp_msg = format!("ERROR: Processing JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );
                let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

                resp_json_message
            },
        };

        // Set the answer to the associated task and flag it as completed
        TaskListManager::current().set_answer_completed(json_message1.execution_id, output_json_message);
    }

    // Stop or Kill all running modules
    ModuleManager::current().kill_all_modules().await;

    // Stop the other loops
    stop_all_loops( );

    info!("**** Stopping MAIN control loop ");
}

async fn process_message(in_json_message: &Value) -> Result<String, String> 
{
    // Create a separated thread and process the message
    //let handle = thread::spawn(|| {
        // Check Authorization
        let tmp_auth_key = String::from( in_json_message["authentication_key"].as_str().unwrap() );
        if check_authorization( &tmp_auth_key ) == false {
            error!("Non authorized. Invalid authentication key: {}", tmp_auth_key);
            return Ok( String::from("Not authorized") );
        } 
        
        // Check license type or demo

        // Record the IP, user, etc for statistics purposes
        
        
        // Process the message based on its code
        process_a_message(in_json_message).await
        
    
    //});

    //let final_error = handle.join();
    
    //if let Err(e) = final_error {
    //    error!("Error processing the message: {:?}", e);
    //}

    //Ok(())
}

async fn process_a_message(in_json_message: &Value) -> Result<String, String> {
    debug!("Received message code: {}", in_json_message["msg_code_id"].as_str().unwrap() );

    match in_json_message["msg_code_id"].as_str() {
        Some("test") => {
            println!("This is a test");
        },

        // === HTTP MESSAGES ==========================

        /*
         * Orbit Propagation 
         * 
         * Check that the user has an account
         * Parameters:
         *  - username. Account user name
         *  - password. Hash encoded (SHA1 256)
         * 
         * It will read the User details from the DB
         * Encode the password read from the DB with SHA1 256
         * If both password match, then it will grant access to the API
         *   It will create the JWT and return it
         * 
         * If OK, it will return a JWT token with an expiration date of 1 day
         * If not, it will return an error
         */
        Some("orb_propagation") => {
            // TODO: fix the error handling
            return Ok( ModuleManager::current().call_module(&in_json_message).await.unwrap() );
        },



        // === INTERNAL MESSAGES ==========================
        /*
         * Process the answer from the module. Subscribe; Orb Propagation 
         */      
        Some("sub_orb_propagation_answer") => {
            // TODO fix the error handling
            ModuleManager::current().handle_module_answer(&in_json_message).await.unwrap();
        },

        _ => { println!("Unknown message code: {}", in_json_message["msg_code_id"].as_str().unwrap() );
               error!("Unknown message code: {}. IGNORED", in_json_message["msg_code_id"].as_str().unwrap());
             }
    };

    Ok( String::from("No error") )
}

/**
 * It does receives messages via a Nen Nanomsg socket.
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
        Ok(s) =>  { info!("Socket BUS to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create FDS BUS socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = bus_control_socket.listen( tmp_config_data.fds_nng_bus_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to FDS BUS. Address: {}", tmp_config_data.fds_nng_bus_address);

    // Set socket timeout
    //bus_control_socket.set_opt::<nng::options::RecvTimeout>( Some(Duration::from_millis(100)) ).unwrap(); 

   

    // Create REP Socket
    // --------------------------------------
    let rep_control_socket = Socket::new(Protocol::Rep0);
    let rep_control_socket = match rep_control_socket  {
        Ok(s) =>  { info!("Socket REP to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create FDS REP socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = rep_control_socket.listen( tmp_config_data.fds_nng_rep_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to FDS REP. Address: {}", tmp_config_data.fds_nng_rep_address);


    // Create SUB Socket
    // ---------------------------------------------------
    let sub_control_socket = Socket::new(Protocol::Sub0);
    let sub_control_socket = match sub_control_socket  {
        Ok(s) =>  { info!("Socket SUB to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create FDS SUB socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = sub_control_socket.listen( tmp_config_data.fds_nng_sub_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to FDS SUB Address: {}", tmp_config_data.fds_nng_sub_address);

    // Set socket timeout
    //sub_control_socket.set_opt::<nng::options::RecvTimeout>( Some(Duration::from_millis(100)) ).unwrap(); 




    // Main control loop REQ socket
    let main_control_socket = Socket::new(Protocol::Push0);
    let main_control_socket = match main_control_socket  {
        Ok(s) =>  { 
            info!("Socket to Main Control correctly created ");
            s
        },
        Err(e) => {
            error!("Unable to create Main Control REQ socket. Error: {}", e.to_string());
            return;
        },
    };

    //let controller_address;

    // unsafe {
    //     controller_address = GLOBAL_DATA.nng_ip_address.clone();
    // }
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
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to Main Control server. Address: {}", controller_address);

    let mut done_flag = false;
    
    while done_flag == false {
        // unsafe {
            //     if GLOBAL_DATA.exit_flag == true {
                //         done_flag = true;
                //         debug!("NNG loop exiting");
                //         continue;
                //     }
                // }
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
            //if e != Error::TimedOut {
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
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
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

            // NOTE: I do not this is going to work
            // FIXME

            // Send the answer back
            let output_msg = block_on( forward_message_internal(&main_control_socket, &payload) );
            let output_msg = match output_msg {
                Ok(o) => o,
                Err(e) => {
                    let error_msg = format!("Error when forwarding JSON message. Error: {}", e);
                    error!("{}", error_msg);

                    build_api_answer_str_json(true, error_msg.as_str(), "")
                },
            };
            
            let tmp_message = Message::from(output_msg.as_bytes());

            // Send the message to the main control loop
            // Message is a JSON
            debug!("Sending REP message: {}", output_msg );

            let status = rep_control_socket.send(tmp_message);
            if let Err(e) = status {
                error!("Error sending reply back to the requester: {:?}", e );
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


// async fn index(req: HttpRequest) -> actix_web::Result<fs::NamedFile> {
// //async fn index(req: HttpRequest) -> impl Responder {
//     info!("   *** Index");

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
// }


/**
 * Return the status of the FDS module
 * Status can be; None, Running, Stopped
 */
async fn get_status() -> impl Responder 
{
    info!("   *** Get Status");

    // let tmp_status;

    // unsafe {
    //     tmp_status = GetStatusMessageResponse {
    //         status: GLOBAL_DATA.fds_status.to_string(),
    //     };
    // }
    //let tmp_data = in_global_data.data.read().unwrap();
    let tmp_data = GLOBAL_DATA.read().unwrap();
    let tmp_status = tmp_data.fds_status.to_string();

    return HttpResponse::Ok().json( tmp_status );
}

/**
 * Return the current version of the API
 */
async fn get_version() -> impl Responder 
{
    info!("   *** Get Version");

    // Read-only. So, it should be fine
    let tmp_version = GetVersionMessageResponse { version : FDSAAS_VERSION.to_string(), };

    return HttpResponse::Ok().json( tmp_version );
}

/**
 * Return the descrption of the selected operation.actix_files
 * It describes how to use the operation
 */
async fn api_usage(in_operation : web::Path<String>) -> impl Responder 
{
    info!("   *** Get API Usage");

    let mut  usage_msg : String;

    match in_operation.as_str() {
        "register" => {
            usage_msg = String::from("Register a new user");
        },

        "create_mission" => {
            usage_msg = String::from("Create a new Mission");
        },

        "create_satellite" => {
            usage_msg = String::from("Create a new Satellite associated to a mission");
        },

        "create_ground_station" => {
            usage_msg = String::from("Create a new Ground Station");
        },

        "orb_propagation" => {
            usage_msg = String::from("Orbit Propagation \n");
            usage_msg.push_str("Parameters: \n");
            usage_msg.push_str("  start_time = Start time");
        },

        _ => { 
            usage_msg = format!("Unknown operation name: {}", in_operation);
        }
    };

    let resp_json_message = build_api_answer_str_json(false, "", usage_msg.as_str());

    return HttpResponse::Ok().json( resp_json_message );
}


/**
 * Forward a message to the Main Control loop and wait for the answer
 * 
 */
async fn forward_message_internal(in_socket: &Socket, in_payload: &String) -> Result<String, String> 
{
    debug!("INT FORWARD sending message: {}", in_payload);

    

    // Add the new task to Task List   
    // let task_manager : &mut TaskListManager;
    // {
        //     let global_data = in_global_data.lock().unwrap();
        //     &mut match global_data.tasks_manager {
            //         Some(tm) => tm,
            //         None => {
                //             error!("Task manager has not been correctly created. Aborting");
                //             return Err( String::from("Task manager has not been correctly created. Aborting") );
                //         },
                //     };
                // }
                
    let mut execution_id = TaskListManager::current().add_task( );

    // Test
    execution_id = TaskListManager::current().add_task( );
    execution_id = TaskListManager::current().add_task( );
    execution_id = TaskListManager::current().add_task( );

    // Create an internall message that contains the execution id
    let internal_message = InternalMessage {
        execution_id:  execution_id,
        payload:       in_payload.clone(),
    };

    let tmp_message = serde_json::to_string(&internal_message).unwrap();

    // Send the message to the main control loop
    let tmp_message = Message::from(tmp_message.as_bytes());
    // Message is a JSON
    
    let status = in_socket.send(tmp_message);
    if let Err(e) = status {
        error!("Error sending HTTP request to the Main Control: {:?}", e );
        return Err( String::from("Error sending message to Main Control loop") );
    }
        
    // Wait for answer
    TaskListManager::current().wait_for_task(execution_id).await.unwrap();

    // Get answer
    let http_output = TaskListManager::current().get_answer(execution_id);
    let http_output = match http_output {
        Ok(m) => m,
        Err(e) => {
            return Err(e);
        },
    };

    
    // // Receive the answer and process it
    // // Message is a JSON    
    // let recv_message = in_socket.recv();
    // let recv_message = match recv_message {
    //     Ok(m) => m,
    //     Err(e) => {
    //         error!("Unable to receive message from main control loop. Error: {}", e.to_string());
    //         return Err( String::from("Error receiving message to Main Control loop") );
    //     },
    // };
    // let tmp_output = recv_message.as_slice().to_vec();

    // let http_output = String::from_utf8(tmp_output).unwrap();
    debug!("REPLYING with received message: {}", http_output);

    return Ok( http_output );
}


/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn forward_message(in_payload: String) -> impl Responder 
{
    debug!("HTTP FORWARD sending message: {}", in_payload );

    // Create a socket with the Main Control loop
    let tmp_control_socket;

    // unsafe {
    //     tmp_control_socket = GLOBAL_DATA.nng_socket.clone();
    // }
    
    {
        let tmp_data = GLOBAL_DATA.read().unwrap();
        tmp_control_socket = tmp_data.nng_socket.clone();
    }

    let tmp_control_socket = match tmp_control_socket {
        Some(s) => s,
        None => { 
            // Create Socket, if it does not exist
            let new_control_socket = Socket::new(Protocol::Push0);
            let new_control_socket = match new_control_socket  {
                Ok(s) =>  { 
                    info!("Socket to Main Control correctly created ");
                    s
                },
                Err(e) => {
                    error!("Unable to create Main Control REP socket. Error: {}", e.to_string());
                    return HttpResponse::InternalServerError().body( e.to_string() )
                },
            };

            let controller_address;

            // unsafe {
            //     controller_address = GLOBAL_DATA.nng_ip_address.clone();
            // }
            
            {
                let tmp_data = GLOBAL_DATA.read().unwrap();
                controller_address = tmp_data.nng_ip_address.clone();
            }

            // Start transmitting
            let unused_result = new_control_socket.dial( controller_address.as_str() );
            if let Err(e) = unused_result {
                error!("Error when starting listening the socket: {}", e );
                return HttpResponse::InternalServerError().body( e.to_string() )
            }
            info!("Correctly connected to Main Control server. Address: {}", controller_address);

            // Store the socket
            // unsafe {
            //     GLOBAL_DATA.nng_socket      = Some( new_control_socket.clone() );
            // }
            {
                //let mut tmp_data = GLOBAL_DATA.write().unwrap();

                let mut tmp_data = match GLOBAL_DATA.try_write() {
                    Ok(d) => d,
                    Err(e) => {
                        error!("Unable to acquire RwLock.write. Error: {}", e);
                        return HttpResponse::InternalServerError().body( e.to_string() );
                    }
                };
                
                tmp_data.nng_socket = Some( new_control_socket.clone() );
            }
            
            new_control_socket
        },
    };

    debug!("Step 4");

    // Create a new task
    let http_output = forward_message_internal(&tmp_control_socket, &in_payload).await;

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

fn usage() 
{
    println!("Incomplete Worlds (c) 2020");
    println!("Flight Dynamics System (FDS) as a Service - HTTP API");
    println!("");
    println!("Usage:    main   config_file_name");
    println!("");
}



// ================================================================
// *
// *  M  A  I  N
// *
// ================================================================
#[actix_rt::main]
async fn main() -> std::io::Result<()> 
{
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 2 {
        usage();
        return Ok(());
    }
    
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

    //let new_task_manager = Arc::new( TaskListManager::new() );


    
    // // Data shared between all threads
    // let GLOBAL_DATA1 : GlobalData = GlobalData {
    //     fds_status :           EnumStatus::NONE,
    //     nng_ip_address :       String::new(),
    //     nng_socket :           None,
    //     exit_flag :            false, 
    //     //tasks_manager:         Some( TaskListManager::new() ),
    //     //tasks_manager:         Arc::clone(&new_task_manager),
    // };
    
    // let HTTP_GLOBAL_DATA : HttpGlobalData = HttpGlobalData {
    //     data :    Mutex::new(GLOBAL_DATA1),
    // };
    // let HTTP_GLOBAL_DATA = Mutex::new(GLOBAL_DATA1);
    
    // unsafe {
    //     //let mut gdata = HTTP_GLOBAL_DATA.data.write().unwrap();
    //     let mut gdata = HTTP_GLOBAL_DATA.lock().unwrap();

    //     gdata.fds_status      = EnumStatus::RUNNING;
    //     gdata.nng_ip_address  = config_variables.fds_int_req_address.clone();
    // }

    // let arc_global_data      = Arc::new( HTTP_GLOBAL_DATA );
    // //let http_global_data     = Arc::clone( &arc_global_data );
    // let nng_global_data      = Arc::clone( &arc_global_data );
    // let main_global_data     = Arc::clone( &arc_global_data );

    {
        let mut tmp_data = GLOBAL_DATA.write().unwrap();

        tmp_data.fds_status      = EnumStatus::RUNNING;
        tmp_data.nng_ip_address  = config_variables.fds_int_req_address.clone();
    }


    // Start http server in a separated thread
    // Channel for retrieving the http server variable
    let (tx, rx) = mpsc::channel();

    let _http_thread = thread::spawn(move || {
        let sys = System::new("http-server");

        //let http_global_data1 = web::Data::new( http_global_data  );

        let srv = HttpServer::new(move || {
            App::new()
    
            // limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit( MAX_SIZE_JSON ))
    
            // Pass data to the handler. It makes a copy
            //.data( http_global_data1.clone())
            //.app_data( http_global_data1.clone() )

            .service(
                web::scope("/fdsaas")
                    .default_service(
                        web::route()
                        .to(|| HttpResponse::MethodNotAllowed()),
                    )

                    // .route("/", web::get().to( index ) )
                    // .route("/index.html", web::get().to( index ) )
                    .route("/api/exit", web::get().to(forward_message))


                    .route("/api/{operation}/usage", web::get().to(api_usage))

                    // REGISTER
                    // ---------------------------------
                    .route("/api/register", web::put().to(forward_message))

                    // GET VERSION
                    // ---------------------------------
                    .route("/api/version", web::get().to(get_version))

                    // GET STATUS
                    // ---------------------------------
                    .route("/api/status", web::get().to(get_status))

                    // Create a Mission
                    // ---------------------------------
                    .route("/api/create_mission", web::post().to(forward_message))

                    // Create a Satellite
                    // ---------------------------------
                    .route("/api/create_satellite", web::post().to(forward_message))

                    // Create a Ground Station
                    // ---------------------------------
                    .route("/api/create_ground_station", web::post().to(forward_message))

                    // PROPAGATE AN ORBIT
                    // ---------------------------------
                    .route("/api/orb_propagation", web::get().to(forward_message))

                    // Execute a plain GMAT script
                    // ---------------------------------
                    .route("/api/execute_script", web::get().to(forward_message))
            )
            
            // Root URL
            // work, but serves only index.html
            .service( fs::Files::new("/", "WebContent").index_file("index.html") )
        })
        .bind(http_address)?
        .run();

        let _ = tx.send(srv);
        sys.run()
    });

    let srv = rx.recv().unwrap();

    //let tmp_config_variables1 = config_variables.clone();
    //let tmp_config_variables3 = config_variables.clone();
    {
        let mut tmp_config_data = CONFIG_VARIABLES.write().unwrap();

        *tmp_config_data = config_variables;
    }

    // NNG (BUS, REQ, SUB) control loop
    let nng_thread = thread::spawn(move || {
        nng_control_loop();
    });

    
    // Main control loop thread
    //let main_thread = thread::spawn(|| { 
        main_control_loop().await;
    //}); 

    nng_thread.join().unwrap();
    
    // Small sleep
    thread::sleep(Duration::from_secs(1));

   // main_thread.join().unwrap();



    // pause accepting new connections
    //srv.pause().await;
    // resume accepting new connections
    //srv.resume().await;
    // Gratecul Stop of server
    srv.stop(true).await;

    now = Utc::now();
    info!("**********************************");
    info!("Finishing FDS Server: {}", now.naive_utc());   
    info!("**********************************");

    Ok(())
}
