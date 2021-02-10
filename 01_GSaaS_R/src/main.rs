/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
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


// Common functions
mod common;
use common::{ read_config_json, config_log, ConfigVariables };





// limit the maximum amount of data that server will accept
const MAX_SIZE_JSON : usize =  262_144;

const GSAAS_VERSION : &str = "0.1";

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


#[derive(Serialize, Deserialize, Debug)]
pub struct GetVersionMessageResponse {
    pub version: String,
}

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



struct GlobalData {
    gs_status :           EnumStatus,
    nng_ip_address :       String,
    nng_socket :           Option<nng::Socket>,
    exit_flag :            bool,
    //tasks_manager:         Option< TaskListManager >,
   // tasks_manager:         Arc< TaskListManager >,
}

// struct HttpGlobalData {
//     data:                  RwLock< GlobalData >,
// }

// Store the Global Data in the Head
// static mut GLOBAL_DATA : GlobalData = GlobalData {
//     gs_status :           EnumStatus::NONE,
//     nng_ip_address :       String::new(),
//    // nng_socket :           None,
//     exit_flag :            false, 
//     //tasks_manager:         None,
// };







//
// ====================================================================
// ====================================================================
// 
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
async fn get_status(in_global_data: web::Data<GlobalData>) -> impl Responder {
    info!("   *** Get Status");

    // let tmp_status;

    // unsafe {
    //     tmp_status = GetStatusMessageResponse {
    //         status: GLOBAL_DATA.fds_status.to_string(),
    //     };
    // }
    let tmp_status = in_global_data.gs_status.to_string();

    return HttpResponse::Ok().json( tmp_status );
}

/**
 * Return the current version of the API
 */
async fn get_version() -> impl Responder {
    info!("   *** Get Version");

    // Read-only. So, it should be fine
    let tmp_version = GetVersionMessageResponse { version : GSAAS_VERSION.to_string(), };

    return HttpResponse::Ok().json( tmp_version );
}

/**
 * Return the descrption of the selected operation.actix_files
 * It describes how to use the operation
 */
async fn api_usage(in_operation : web::Path<String>) -> impl Responder {
    info!("   *** Get API Usage");

    let mut  usage_msg : String;

    match in_operation.as_str() {
        "tm_decoder/create" => {
            usage_msg = String::from("Create and configure the TM Decoder.\n");
            usage_msg.push_str("Parameters: \n");
            usage_msg.push_str("  start_time = Start time");

        },

        "tm_decoder/connect" => {
            usage_msg = String::from("Start the TM Decoder.\n");
            usage_msg.push_str("It will connect to the ground station (IP/port) and process the incoming data frames\n");
            usage_msg.push_str("After that, it will send the processed frames (JSON, CSV) to the destination (IP/port)\n");
            
        },

        "tm_decoder/disconnect" => {
            usage_msg = String::from("Stop the TM Decoder.\n");
            usage_msg.push_str("It will disconnect from the ground station (IP/port)\n");
        },

        "tm_decoder/shutdown" => {
            usage_msg = String::from("Shutdown the TM Decoder.\n");
            usage_msg.push_str("It will disconnect from the ground station (IP/port)\n");
        },

        _ => { 
            usage_msg = format!("Unknown operation name: {}", in_operation);
        }
    };

    let resp_json_message = build_api_answer_str_json(false, "", usage_msg.as_str());

    return HttpResponse::Ok().json( resp_json_message );
}




/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn tm_decoder_create(in_payload: String, in_global_data: web::Data<GlobalData>) -> impl Responder 
{
    debug!("TM Decoder Create and Configure");

    let http_output = String::from("empty");
 
    return HttpResponse::Ok()
                .content_type("application/json")
                .body( http_output );
}

/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn tm_decoder_connect(in_payload: String, in_global_data: web::Data<GlobalData>) -> impl Responder 
{
    debug!("TM Decoder. START Time: {}", Utc::now().naive_utc() );

    let http_output = String::from("empty");
 
    return HttpResponse::Ok()
                .content_type("application/json")
                .body( http_output );
}

/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn tm_decoder_disconnect(in_payload: String, in_global_data: web::Data<GlobalData>) -> impl Responder 
{
    
    debug!("TM Decoder. STOP Time: {}", Utc::now().naive_utc() );

    let http_output = String::from("empty");
 
    return HttpResponse::Ok()
                .content_type("application/json")
                .body( http_output );
}

/**
 * Forward a message to the main control loop and wait for the answer
 */
async fn tm_decoder_shutdown(in_payload: String, in_global_data: web::Data<GlobalData>) -> impl Responder 
{
    
    debug!("TM Decoder. STOP Time: {}", Utc::now().naive_utc() );

    let http_output = String::from("empty");
 
    return HttpResponse::Ok()
                .content_type("application/json")
                .body( http_output );
}

fn usage() {
    println!("Incomplete Worlds (c) 2020");
    println!("Ground Segment (GS) as a Service - HTTP API");
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
async fn main() -> std::io::Result<()> {    
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 2 {
        usage();
        return Ok(());
    }
    
    let mut now: DateTime<Utc> = Utc::now(); 

    println!("**********************************");
    println!("Initializing GS as a Service - HTTP API : {}", now.naive_utc() );
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
    info!("Initializing GS as a Service - HTTP API : {}", now.naive_utc() );
    info!("**********************************");

    // External HTTP Address. It will listen for HTTP requests coming from this address
    let http_address = config_variables.gsaas_http_address.clone();
   
    info!("Listening HTTP IP Address: {}", http_address);

    
    // Create PUSH socket
    let decoder_socket = Socket::new(Protocol::Push0);
    let decoder_socket = match decoder_socket  {
        Ok(s) =>  { 
            info!("Socket to TM Decoder correctly created ");
            s
        },
        Err(e) => {
            error!("Unable to create TM Decoder PUSH socket. Error: {}", e.to_string());
            return Ok(());
        },
    };

    let decoder_address = config_variables.decoder_nng_push_address.clone();

    // Start transmitting
    let unused_result = decoder_socket.dial( decoder_address.as_str() );
    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return Ok(());
    }
    info!("Correctly connected to TM Decoder server. Address: {}", decoder_address);



    // // Data shared between all threads
    let mut GLOBAL_DATA : GlobalData = GlobalData {
        gs_status :           EnumStatus::NONE,
        nng_ip_address :      config_variables.decoder_nng_push_address.clone(),
        nng_socket :          Some(decoder_socket),
        exit_flag :           false, 
    };
    

    let global_data = web::Data::new(  GLOBAL_DATA );

    //let srv = 
    HttpServer::new(move || {
            App::new()
    
            // limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit( MAX_SIZE_JSON ))
    
            // Pass data to the handler. It makes a copy
            //.data(global_data.clone())
            .app_data( global_data.clone() )

            .service(
                web::scope("/gsaas")
                    .default_service(
                        web::route()
                        .to(|| HttpResponse::MethodNotAllowed()),
                    )

                    // .route("/", web::get().to( index ) )
                    // .route("/index.html", web::get().to( index ) )

                    // GENERAL
                    // ---------------------------------
                    .route("/api/version", web::get().to(get_version))
                    .route("/api/status", web::get().to(get_status))

                    // TM DECODER
                    // ---------------------------------
                    .route("/api/tm_decoder/create", web::get().to(tm_decoder_create))

                    .route("/api/tm_decoder/connect", web::get().to(tm_decoder_connect))

                    .route("/api/tm_decoder/disconnect", web::get().to(tm_decoder_disconnect))

                    .route("/api/tm_decoder/shutdown", web::get().to(tm_decoder_shutdown))
                    
                    // General Usage 
                    // ---------------------------------
                    .route("/api/{operation}/{operation1}/usage", web::get().to(api_usage))
            )
            
            // Root URL
            // work, but serves only index.html
            .service( fs::Files::new("/", "WebContent").index_file("index.html") )
        })
        .bind(http_address)?
        .run()
        .await.unwrap();
        
    // pause accepting new connections
    //srv.pause().await;
    // resume accepting new connections
    //srv.resume().await;
    // Gratecul Stop of server
    //srv.stop(true).await;

    now = Utc::now();
    info!("**********************************");
    info!("Finishing GS Server: {}", now.naive_utc());   
    info!("**********************************");

    Ok(())
}
