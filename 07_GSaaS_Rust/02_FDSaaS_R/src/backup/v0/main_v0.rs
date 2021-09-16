//#![deny(warnings)]
//#![deny(unused_imports)]


/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * It will implement the entry point and the REST API
 */


use std::env;
use std::thread;
use std::sync::mpsc;
// use std::sync::mpsc::channel;
// use std::collections::HashMap;
//use std::error::Error;
use std::result::Result;

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json;

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
use actix_files::{ NamedFile };
// use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_rt::System;

// New Nanomsg
use nng::*;

// Messages
mod fds_messages;
use fds_messages::*;
//use fds_messages::{ApiMessage, build_api_message, build_api_message_str, build_api_answer, build_api_answer_str, build_api_answer_str_json};

mod authorization_manager;
use authorization_manager::check_authorization;

mod modules_manager;
use modules_manager::{ execute_module, execute_module1 };

// Common functions
mod common;
use common::{ read_config, config_log };

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

#[derive(Clone)]
struct GlobalData {
    // fds_socket:        Socket,
    // control_socket:    Socket,
    // request_channel:   mpsc::Sender<ApiMessage>,
    // reply_channel:     mpsc::Receiver<ApiMessage>,
    fds_status:        EnumStatus,
    //nng_ip_address:    &'static str,
    nng_ip_address:    String,
}

//  impl Clone for AppState {
//      fn clone(&self) -> Self {
//         let tmp_version = self.api_version;
//         let tmp_status = self.status;

//         let xx = AppState {
//             api_version : tmp_version.to_string(),
//             status : tmp_status.to_string(),
//         };

//         return xx;
//            *self
//     }
// }

// impl Copy for AppState {
//     fn copy(&self, &sec) -> AppState {
//         let xx = AppState(
//             api_version :   String::from("0.1"),
//             status :        String::from("None"),
//         );
//         xx
//     }
//}

// static mut global_AppState : AppState = AppState {
//     api_version :   String::from("0.1"),
//     status :        String::from("None"),
// };


static mut GLOBAL_DATA : GlobalData = GlobalData {
    //fds_socket: tmp_socket,
    //fds_socket:       Socket::new(Protocol::Req0)?,    Not needed  
    //control_socket:   Socket::new(Protocol::Req0)?,    Not needed
    // request_channel:  to_main,
    // reply_channel:    from_http,
    
    fds_status:       EnumStatus::RUNNING,
    nng_ip_address:   String::new(),
};



//
// ====================================================================
// ====================================================================
// 

/**
 * It does receives messages from the HTTP thread and send the replies back to
 * HTTP
 */
fn main_control_loop(in_nng_address: &String) {
    
    // Create Socket
    let rep_control_socket = Socket::new(Protocol::Rep0);
    let rep_control_socket   = match rep_control_socket  {
        Ok(s) =>  { info!("Socket to FDS server correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create main control REP socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = rep_control_socket.listen( in_nng_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to Main Control (REP). Address: {}", in_nng_address);

    let mut done_flag = false;

    while done_flag == false {
        // Read a message
        // ------------------------------------------
        // let input_msg = in_from_main.recv();
        // debug!("Received message: {}", input_msg.unwrap().to_string());
        
        // if let Err(e) = input_msg {
        //     warn!("Error when receiving a message: {}", e.to_string());
        //     warn!("Message will be IGNORED");
        //     continue;
        // }

        // This code shall be used when nanomsg socket is being used
        let input_msg = rep_control_socket.recv();
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        if let Err(e) = input_msg {
            //if e.kind() == TryRecvError::Disconnected {
                error!("Error when receiving a message: {}", e );
                // End of the loop
                break;
            //}
        }
        
        // As u8[]
        let json_buffer = input_msg.unwrap();

        debug!("Received message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );

        // Decode JSON
/*
        let json_message = serde_json::from_slice( json_buffer.as_slice() );
        let json_message : serde_json::Value  = match json_message {
            Ok(msg) => msg,
            Err(e) => {
                let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );
                let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");
                
                let mut socket_m = nng::Message::from(resp_json_message.as_bytes());
                
                rep_control_socket.send(socket_m);
                continue;
            },
        };
        
        // If end of processing, then break the loop
        if json_message["msg_code_id"].is_null() == true {
            let tmp_msg = format!("ERROR: No msg code found. IGNORED");
            error!("{}", tmp_msg.as_str() );
            
            let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

            let mut socket_m = nng::Message::from(resp_json_message.as_bytes());

            rep_control_socket.send(socket_m);
            continue;
        } 

        if json_message["authentication_key"].is_null() == true {
            let tmp_msg = format!("ERROR: No Authentication key. IGNORED");
            error!("{}", tmp_msg.as_str() );

            let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");

            let mut socket_m = nng::Message::from(resp_json_message.as_bytes());

            rep_control_socket.send(socket_m);
            continue;
        }
        
        if json_message["msg_code_id"] == "end" {
            info!("Shutting down the FDS server");
            done_flag = true;
            continue;
        }      
        
        // Process message
        let response_message = process_message(&json_message);

        let json_message : String  = match response_message {
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
*/



        // Decode JSON
        let json_message = serde_json::from_slice(json_buffer.as_slice());
        let json_message : ApiMessage  = match json_message {
            Ok(msg) => msg,
            Err(e) => {
                let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                error!("{}", tmp_msg.as_str() );
                let resp_json_message = build_api_answer_str_json(true, tmp_msg.as_str(), "");
                
                let mut socket_m = nng::Message::from(resp_json_message.as_bytes());
                
                rep_control_socket.send(socket_m);
                continue;
            },
        };

        // Process message
        let response_message = process_message1(&json_message);

        let json_message : String  = match response_message {
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

        // Send back the answer
        let mut socket_m = nng::Message::from(json_message.as_bytes());

        rep_control_socket.send(socket_m);
    }

    // Shutting down the server
    //rep_control_socket.close();
}

fn process_message(in_json_message: &serde_json::Value) -> Result<String, String> {
    // Create a separated thread and process the message
    //let handle = thread::spawn(|| {
        let authorization_key : String = in_json_message["authentication_key"].to_string();

        // Check Authorization
        if authorization_manager::check_authorization( &authorization_key ).unwrap() == false {
            error!("Non authorized");
            Ok( String::from("Not authorized") )

        } else {
            // Process the message based on its code
            process_a_message(in_json_message)
        }
    
    //});

    //let final_error = handle.join();
    
    //if let Err(e) = final_error {
    //    error!("Error processing the message: {:?}", e);
    //}

    //Ok(())
}




fn process_a_message(in_json_message: &serde_json::Value) -> Result<String, String> {
    let tmp_code_id = in_json_message["msg_code_id"].as_str().unwrap();
    
    debug!("Received message code: {}", tmp_code_id);

    match tmp_code_id {
        "test" => {
            println!("This is a test");
        },

        "orb_propagation" => {
            // TODO fix the error handling
             execute_module(&in_json_message).unwrap();
        },
                  

        _ => { println!("Unknown message code: {}", tmp_code_id);
               error!("Unknown message code: {}. IGNORED", tmp_code_id);
             }
    };

    Ok( String::from("No error") )
}

fn process_message1(in_json_message: &ApiMessage) -> Result<String, String> {
    // Create a separated thread and process the message
    //let handle = thread::spawn(|| {
        // Check Authorization
        if check_authorization( &in_json_message.authentication_key ).unwrap() == false {
            error!("Non authorized");
            Ok( String::from("Not authorized") )

        } else {
            // Process the message based on its code
            process_a_message1(in_json_message)
        }
    
    //});

    //let final_error = handle.join();
    
    //if let Err(e) = final_error {
    //    error!("Error processing the message: {:?}", e);
    //}

    //Ok(())
}

fn process_a_message1(in_json_message: &ApiMessage) -> Result<String, String> {
    // let tmp_code_id = in_json_message["msg_code_id"].as_str().unwrap();
    
    debug!("Received message code: {}", in_json_message.msg_code_id);

    match in_json_message.msg_code_id.as_str() {
        "test" => {
            println!("This is a test");
        },

        "orb_propagation" => {
            // TODO fix the error handling
             execute_module1(&in_json_message).unwrap();
        },
                  

        _ => { println!("Unknown message code: {}", in_json_message.msg_code_id);
               error!("Unknown message code: {}. IGNORED", in_json_message.msg_code_id);
             }
    };

    Ok( String::from("No error") )
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
 * Return the current version of the API
 */
async fn get_version() -> impl Responder {
    info!("   *** Get Version");

    // Read-only. So, it should be fine
    let tmp_version = GetVersionMessageResponse { version : FDSAAS_VERSION.to_string(), };

    return HttpResponse::Ok().json( tmp_version );
}

/**
 * Return the status of the FDS module
 * Status can be; None, Running, Stopped
 */
// Work
// async fn get_status(in_payload: String,
//    in_global_data: web::Data<GlobalData>) -> impl Responder {

// async fn get_status(in_message: web::Json<ApiMessage>,
//      in_global_data: web::Data<GlobalData>) -> impl Responder {

async fn get_status() -> impl Responder {
    info!("   *** Get Status");

    let tmp_status;

    unsafe {
        tmp_status = GetStatusMessageResponse {
            status: GLOBAL_DATA.fds_status.to_string(),
        };
    }

    return HttpResponse::Ok().json( tmp_status );
}

// Orbit Propagation
/**
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
// async fn orb_propagation(in_message: web::Json<fds_messages::OrbPropagationMessage>, 
//     in_global_data: web::Data<GlobalData>) -> impl Responder {
async fn orb_propagation(in_payload: String) -> impl Responder {
    info!("   *** Orbit Propagation");

    // Create Socket
    let tmp_control_socket = Socket::new(Protocol::Req0);
    let tmp_control_socket   = match tmp_control_socket  {
        Ok(s) =>  { info!("Socket to Main Control correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create Main Control REP socket. Error: {}", e.to_string());
            return HttpResponse::InternalServerError().body("Error sending message");
        },
    };

    let controller_address;

    unsafe {
        controller_address = GLOBAL_DATA.nng_ip_address.clone();
    }

    // Start listening
    let unused_result = tmp_control_socket.dial( controller_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return HttpResponse::InternalServerError().body("Error sending message");
    }
    info!("Correctly connected to Main Control server. Address: {}", controller_address);


    // Send to main control loop
    let tmp_message = Message::from(in_payload.as_bytes());


    // Send the message to the main control loop
    // Message is a JSON
    debug!("sending message: {}", in_payload );

    let status = tmp_control_socket.send(tmp_message);
    if let Err(e) = status {
        error!("Error sending HTTP request to the Main Control: {:?}", e );
        return HttpResponse::InternalServerError().body("Error sending message");
    }


    // Receive the answer and process it
    // Message is a JSON
    let tmp_output = tmp_control_socket.recv().unwrap().as_slice().to_vec();
    
    let http_output = String::from_utf8(tmp_output).unwrap();
    debug!("received message: {}", http_output);
    
    
    // Create response and set content type
    return HttpResponse::Ok()
            .content_type("application/json")
            .body( http_output );
}

fn usage() {
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
async fn main() -> std::io::Result<()> {     // -> Result<_, _> { // {
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
    let config_variables = read_config(&tmp_config_file_name);
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
    let tmp_log_filename = config_variables["log_filename"].clone();

    let error_code = config_log(&tmp_log_filename);
    if let Err(e) = error_code {
        println!("ERROR: Unable to read the Log Configuration file: {}", tmp_log_filename.as_str());
        println!("{} {}", e.to_string(), e);
        return Ok(());
    }

    // External HTTP Address. It will listen for HTTP requests coming from this address
    let server_address = config_variables["fdsaas_server_address"].clone();
    let server_port    = config_variables["fdsaas_port"].clone();
    let mut ip_address = server_address;
    // Null '~'  value in Yaml
    if server_port != String::from("~") {
        ip_address     = ip_address + &String::from(":") + &server_port;
    }
   
    info!("Listening IP Address: {}", ip_address);

    
    // Connect to the FDS server via Nanomsg 
    let nng_server_address  = config_variables["fds_nng_req_address"].clone();
    let nng_server_port     = config_variables["fds_nng_req_port"].clone();
    // We assume it is not a TCP connection. So, port is not used
    let mut nng_ip_address  = nng_server_address;

    // Null '~'  value in Yaml
    if nng_server_port != String::from("~") {
        nng_ip_address     = nng_ip_address + &String::from(":") + &nng_server_port;
    }

    info!("Nanomsg IP Address: {}", nng_ip_address);

    unsafe {
        GLOBAL_DATA.nng_ip_address = nng_ip_address.clone();
    }

    /*
    let tmp_socket = Socket::new(Protocol::Req0)?;
    info!("Socket to FDS server correctly created ");
    
    tmp_socket.dial( nng_ip_address.as_str() )?;
    info!("Correctly connected to FDS server");
    */

    
       
        // unsafe {
        //     global_AppState = tmpState;
        // };

   

    // Channel for exchanging data between the main thread and the http thread
    // let (to_main, from_main): (mpsc::Sender<ApiMessage>, mpsc::Receiver<ApiMessage>) = mpsc::channel();
    // let (to_http, from_http): (mpsc::Sender<ApiMessageAnswer>, mpsc::Receiver<ApiMessageAnswer>) = mpsc::channel();
    // let (to_main, from_main): (mpsc::Sender<ApiMessage>, mpsc::Receiver<ApiMessage>) = mpsc::channel();
    // let (to_http, from_http): (mpsc::Sender<ApiMessage>, mpsc::Receiver<ApiMessage>) = mpsc::channel();

    // let global_data = GlobalData {
    //     //fds_socket: tmp_socket,
    //     fds_socket:       Socket::new(Protocol::Req0)?,    Not needed  
    //     control_socket:   Socket::new(Protocol::Req0)?,    Not needed
    //     fds_status:       EnumStatus::RUNNING,
    //     // request_channel:  to_main,
    //     // reply_channel:    from_http,

    //     nng_ip_address : String,  vec<u8>,

    // };

    // Start http server in a separated thread
    // Channel for retrieving the http server variable
    let (tx, rx) = mpsc::channel();

    let http_thread = thread::spawn(move || {
        let sys = System::new("http-server");

        let srv = HttpServer::new(move || {
            App::new()
    
            // limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit( MAX_SIZE_JSON ))
    
            // Pass data to the handler. It makes a copy
            //.data(global_data.clone())

            .service(
                web::scope("/fdsaas")
                    .default_service(
                        web::route()
                        .to(|| HttpResponse::MethodNotAllowed()),
                    )

                    // .route("/", web::get().to( index ) )
                    // .route("/index.html", web::get().to( index ) )
            
                    // General
                    // GET VERSION
                    // ---------------------------------
                    // .service(
                    //     web::resource("/version")
                    //         //.wrap(redirect::CheckAuthorization),
                    //         .route(web::get().to(get_version)),
                    // )
                    .route("/api/version", web::get().to(get_version))

                    // GET STATUS
                    // ---------------------------------
                    // .service(
                    //     web::resource("/status")
                    //         //.wrap(redirect::CheckAuthorization),
                    //         .route(web::get().to(get_status)),                         
                    // )
                    .route("/api/status", web::get().to(get_status))
    
                    // PROPAGATE AN ORBIT
                    // ---------------------------------
                    // .service(
                    //     web::resource("/orb_propagation")
                    //         //.wrap(redirect::CheckAuthorization),
                    //         .route(web::get().to(orb_propagation)),
                    // )
                    .route("/api/orb_propagation", web::get().to(orb_propagation))
            )
            
            // Root URL
            // work, but serves only index.html
            .service( fs::Files::new("/", "WebContent").index_file("index.html") )
        })
        .bind(ip_address)?
        .run();

        let _ = tx.send(srv);
        sys.run()
    });

    let srv = rx.recv().unwrap();

    // Main control loop thread
    main_control_loop(&nng_ip_address);

    // Wait for http server thread to finish
    http_thread.join().unwrap();

    // pause accepting new connections
    //srv.pause().await;
    // resume accepting new connections
    //srv.resume().await;
    // stop server
    srv.stop(true).await;

    now = Utc::now();
    info!("**********************************");
    info!("Finishing FDS Server: {}", now.naive_utc());   
    info!("**********************************");

    Ok(())
}


    

