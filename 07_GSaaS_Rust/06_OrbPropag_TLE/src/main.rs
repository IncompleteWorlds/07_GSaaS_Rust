/**
* (c) Incomplete Worlds 2021
* Alberto Fernandez (ajfg)
*
* GS as a Service
* Orbit Propagation - SGP4 - TLE
*
* Receive a HTTP+REST message via the a socket for 
* propagating the orbit of a S/C using the SGP4 algorithm
*/
use std::result::Result;
use std::{env, thread};
use std::time::{Duration};
use std::sync::{Arc, RwLock};
use std::sync::mpsc;
use std::fs;
use std::process;
use futures::executor;


// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Log
use log::{debug, error, info, trace, warn};
// use log::{LevelFilter, SetLoggerError};
// use log4rs::append::console::{ConsoleAppender, Target};
// use log4rs::append::file::FileAppender;
// use log4rs::config::{Appender, Config, Root};
// use log4rs::encode::pattern::PatternEncoder;
// use log4rs::filter::threshold::ThresholdFilter;

// Date & Time
use chrono::{DateTime, Utc};

// Actix Web Server
use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest /*middleware*/};
use actix_web::{error::BlockingError, http::StatusCode};
use actix_files as actixfs;

// Database access and connection pools
// Important: It has to be included in the Root file
#[macro_use]
extern crate diesel;

//#[macro_use]
use lazy_static::lazy_static;

// Components
use common::common::*;
use common::common_messages::*;
use common::http_errors::*;

// Common functions
mod config_tools;
use config_tools::{config_log, read_config_json, ConfigVariables};

mod api_messages;
mod db;
use db::*;
use db::user::*;

// use db::mission::*;
// use db::satellite::*;
// use db::ground_station::*;
// use db::antenna::*;






const ORB_PROPAG_TLE_VERSION : &str = "0.1";


struct GlobalData {
    service_status :           EnumStatus,
    // Exit flag of all loops
    exit_flag :            bool,
    //db_pool:               DbPool,
}

impl GlobalData {
    pub fn new() -> Self 
    {
        GlobalData {
            service_status :       EnumStatus::NONE,
            exit_flag :            false, 
            //db_pool :              establish_connection(),
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






/**
 * Check the input parameters of the REST message
 * Return false - there are no errors
 */
fn check_parameters(in_json_message: &RestRequest) -> Result<bool, RestResponse> {
    // First item to be checked, so, we print the message_id later
    if in_json_message.msg_id.is_empty() == true {
        let tmp_msg = format!("ERROR: Message Id not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        let resp_json_message =
            RestResponse::new_error_msg(String::from("error_response"), String::from("-1"), tmp_msg);

        return Err(resp_json_message);
    }

    if in_json_message.version.is_empty() == true {
        let tmp_msg = format!("ERROR: Version not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        let resp_json_message =
            RestResponse::new_error_msg(String::from("error_response"), in_json_message.msg_id.clone(), tmp_msg);

        return Err(resp_json_message);
    }

    if in_json_message.version != String::from(REST_JSON_VERSION) {
        let tmp_msg = format!("ERROR: Incorrect version number. IGNORED");
        error!("{}", tmp_msg.as_str());

        let resp_json_message =
            RestResponse::new_error_msg(String::from("error_response"), in_json_message.msg_id.clone(), tmp_msg);

        return Err(resp_json_message);
    }

    if in_json_message.msg_code.is_empty() == true {
        let tmp_msg = format!("ERROR: Msg code not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        let resp_json_message =
            RestResponse::new_error_msg(String::from("error_response"), in_json_message.msg_id.clone(), tmp_msg);

        return Err(resp_json_message);
    }

    // authorization_key is not present in Register and Login messages
    if in_json_message.msg_code != "login" && in_json_message.msg_code != "register" {
        if in_json_message.authentication_key.is_empty() == true {
            let tmp_msg = format!("ERROR: Authentication key not found. IGNORED");
            error!("{}", tmp_msg.as_str());

            let resp_json_message = RestResponse::new_error_msg(
                String::from("error_response"),
                in_json_message.msg_id.clone(),
                tmp_msg,
            );

            return Err(resp_json_message);
        }
    }

    if in_json_message.timestamp.is_null() == true {
        let tmp_msg = format!("ERROR: No Timestamp. IGNORED");
        error!("{}", tmp_msg.as_str());

        let resp_json_message = RestResponse::new_error_msg(
            String::from("error_response"),
            in_json_message.msg_id.clone(),
            tmp_msg
        );

        return Err(resp_json_message);
    }

    return Ok(false);
}

/**
 * Return index.html
 */
// async fn index(req: HttpRequest) -> actix_web::Result<fs::NamedFile> {
async fn index(in_request: HttpRequest, in_db: web::Data<db::DbPool>) -> actix_web::Result<actixfs::NamedFile>  { // -> impl Responder
    debug!("   *** Index");

    // Record the HTTP access
    //record_access(&in_request, &in_db);

    Ok( actixfs::NamedFile::open("./WebContent/index.html")?.set_content_type(mime::TEXT_HTML_UTF_8).set_status_code(StatusCode::NOT_FOUND) )

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
async fn not_allowed_method(in_payload: String, in_request: HttpRequest, in_db: web::Data<db::DbPool>) -> HttpResponse 
{
    debug!("Not Allowed Method: {}", in_payload );

    // Record the HTTP access
    //record_access(&in_request, &in_db);

    HttpResponse::MethodNotAllowed().finish()
}
    

/**
 * Return the descrption of the selected operation.actix_files
 * It describes how to use the operation
 */
async fn api_usage(in_operation : web::Path<String>) -> HttpResponse 
{
    info!("   *** Get API Usage");

    let usage_msg : String;

    match in_operation.as_str() {

        "orb_propagation_tle" => {
            usage_msg = fs::read_to_string("doc/orb_propagation_tle.html").expect("Unable to read 'doc/orb_propagation_tle.html' file");
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
 * Return the status of the tools module
 * Status can be; None, Running, Stopped
 */
async fn get_status() -> HttpResponse 
{
    debug!("   *** Get Status");

    let tmp_data = GLOBAL_DATA.read().unwrap();
    // let tmp_status = tmp_data.fds_status.to_string();

    let tmp_status = GetStatusResponseStruct{ status : tmp_data.service_status.to_string() };

    return HttpResponse::Ok().content_type("application/json")
                             .json( tmp_status );
}

/**
 * Return the current version
 */
async fn get_version() -> HttpResponse 
{
    debug!("   *** Get Version");

    let tmp_version = GetVersionResponseStruct{ version: ORB_PROPAG_TLE_VERSION.to_string() };

    return HttpResponse::Ok().content_type("application/json")
                             .json( tmp_version );
}

/**
 * Check the user and password (hash), and if correct generate a JWT token
 */
async fn orb_propagation_tle(in_msg: web::Json<RestRequest>, 
                       in_db_pool: web::Data<db::DbPool>,
                       in_cfg: web::Data<ConfigVariables>) -> Result<HttpResponse, HttpServiceError>
{
    debug!("Orbit Propagation - SGP4 - TLE Input msg: {}", in_msg.to_string());

    // Check minimum set of fields
    if let Err(e) = check_parameters(&in_msg) {
        return Err( HttpServiceError::BadRequest(in_msg.msg_id.clone(), e.to_string()) );
    } 

    let elements = sgp4::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992".as_bytes(),
        "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008".as_bytes(),
    )?;
    let constants = sgp4::Constants::from_elements(&elements)?;

    for hours in 0..24 {
        println!("t = {} min", hours * 60);
        let prediction = constants.propagate((hours * 60) as f64)?;
        println!("    r = {:?} km", prediction.position);
        println!("    ṙ = {:?} km.s⁻¹", prediction.velocity);
    }

    Ok( HttpResponse::Ok().content_type("application/json")
                          .json("{ 'test':'test'}") )



















    // let new_conn = in_db_pool.get().unwrap();
    // let tmp_msg_id = in_msg.msg_id.clone();

    // let res = web::block(move || 
    //     UserDb::login(&new_conn, &in_msg, &in_cfg.secret_key)
    // ).await;
    
    // match res {
    //     Ok(u) => {
    //         Ok( HttpResponse::Ok().content_type("application/json")
    //                           .json(u) )
    //     },

    //     Err(err) => match err {
    //         BlockingError::Error(service_error) => Err(service_error),
    //         BlockingError::Canceled => Err(HttpServiceError::InternalServerError(tmp_msg_id, String::from("Cancelled operation")) ),
    //     },
    // }    
}

/**
 * Stop the server, if the key is correct
 */
async fn stop_fn(in_key : web::Path<String>, stopper: web::Data<mpsc::Sender<()>>) -> HttpResponse 
{
    // Check the key
    if in_key.as_str() == "XYZZY" {
        info!("***TOOLS Main. Leaving");

        // make request that sends message through the Sender
        stopper.send(()).unwrap();
    
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::Ok().content_type("text/plain")
                        .body( String::from("Tools Service is running") )
    }
}

fn usage() 
{
    println!("Incomplete Worlds (c) 2021");
    println!("orbit Propagation - SGP4 - TLE");
    println!("");
    println!("Usage:    main   config_file_name");
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
    fs::write("tools.pid", data).expect("Unable to write 'tools.pid' file");


    let now:  DateTime<Utc> = Utc::now(); 

    println!("**********************************");
    println!("Initializing Orbit Propagation TLE: {}", now.naive_utc() );
    println!("**********************************");

    // Load configuration file
    debug!("Reading the configuration file");

    let tmp_config_file_name = args[1].clone();
    let config_variables = read_config_json(&tmp_config_file_name);
    let config_variables = match config_variables {
        // Just return the variables
        Ok(tmp_variables) => tmp_variables,
        Err(tmp_error) => {
            println!(
                "Unable to read the configuration file: {}",
                tmp_config_file_name.as_str()
            );
            println!("Error: {}", tmp_error);
            return Ok(());
        }
    };

    // Init Log
    let tmp_log_filename = config_variables.config_log_filename.clone();

    let error_code = config_log(&tmp_log_filename);
    if let Err(e) = error_code {
        println!(
            "ERROR: Unable to read the Log Configuration file: {}",
            tmp_log_filename.as_str()
        );
        println!("{} {}", e.to_string(), e);
        return Ok(());
    }

    info!("**********************************");
    info!("Initializing Orbit Propagation TLE Service - HTTP API : {}", now.naive_utc() );
    info!("**********************************");

    // External HTTP Address. It will listen for HTTP requests coming from this address
    let http_address = config_variables.orb_propagation_tle_http_address.clone();
   
    info!("Listening HTTP IP Address: {}", http_address);


    //let db_addr = SyncArbiter::start(num_cpus::get(), move || DatabaseExecutor(pool.clone()));
    // Copy to be transfered to HTTP server
    //let conn_pool_copy = conn_pool.clone();
    let conn_pool_copy = establish_connection();

    // Data shared between all threads
    {
        let mut tmp_data = GLOBAL_DATA.write().unwrap();

        tmp_data.service_status    = EnumStatus::RUNNING;
        // tmp_data.nng_ip_address  = config_variables.fds_int_address.clone();
        //tmp_data.db_pool         = conn_pool;
        //conn_pool_copy             = tmp_data.db_pool.clone();
    }

    debug!("Creating global data");


    let (tx, rx) = mpsc::channel::<()>();

    let srv = HttpServer::new(move || {
        App::new()

        // limit the maximum amount of data that server will accept
        .data(web::JsonConfig::default().limit( common::common::MAX_SIZE_JSON ))

        // Pass data to the handler. It makes a copy
        .data( conn_pool_copy.clone() )

        .data( config_variables.clone() )

        // Stopping the server
        .data( tx.clone() )

        .service(
            web::scope("/fds")
                .default_service(
                    web::route().to(not_allowed_method),
                )

                // GENERAL
                // ---------------------------------
                .route("/{version}/{operation}/usage", web::get().to(api_usage))
                .route("/version", web::get().to(get_version))
                .route("/status", web::get().to(get_status))
                .route("/exit/{key}", web::post().to(stop_fn))


                // MODULE SPECIFIC
                .route("/{version}/orb_propagation_tle", web::get().to(orb_propagation_tle))
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

    debug!("HTTP server created");

    // clone the Server handle
    let server = srv.clone();
    thread::spawn(move || {
        // wait for shutdown signal
        rx.recv().unwrap();

        // stop server gracefully
        executor::block_on(server.stop(true))
    });  
    
    debug!("Thread created");

    srv.await
}
