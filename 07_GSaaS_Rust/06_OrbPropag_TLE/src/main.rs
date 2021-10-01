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

// Date & Time
use chrono::{DateTime, Utc, TimeZone, Duration};

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
use api_messages::*;
use sgp4::Prediction;



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
fn check_parameters(in_json_message: &RestRequest) -> Result<bool, HttpServiceError> 
{
    // First item to be checked, so, we print the message_id later
    if in_json_message.msg_id.is_empty() == true {
        let tmp_msg = format!("ERROR: Message Id not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        return Err( HttpServiceError::BadRequest(String::from("-1"), tmp_msg) );
    }

    if in_json_message.version.is_empty() == true {
        let tmp_msg = format!("ERROR: Version not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        return Err( HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg) );
    }

    if in_json_message.version != String::from(REST_JSON_VERSION) {
        let tmp_msg = format!("ERROR: Incorrect version number. IGNORED");
        error!("{}", tmp_msg.as_str());

        return Err( HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg) );
    }

    if in_json_message.msg_code.is_empty() == true {
        let tmp_msg = format!("ERROR: Msg code not found. IGNORED");
        error!("{}", tmp_msg.as_str());

        return Err( HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg) );
    }

    // authorization_key is not present in Register and Login messages
    if in_json_message.msg_code != "login" && in_json_message.msg_code != "register" {
        if in_json_message.authentication_key.is_empty() == true {
            let tmp_msg = format!("ERROR: Authentication key not found. IGNORED");
            error!("{}", tmp_msg.as_str());

            return Err( HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg) );
        }
    }

    if in_json_message.timestamp.is_null() == true {
        let tmp_msg = format!("ERROR: No Timestamp. IGNORED");
        error!("{}", tmp_msg.as_str());

        return Err( HttpServiceError::BadRequest(in_json_message.msg_id.clone(), tmp_msg) );
    }

    return Ok(false);
}

/**
 * Check the specific parameters of this operation
 * Return false - if there are no errors
 */
 fn check_operation_parameters(in_message: &OrbPropagationTleStruct, in_msg_id: String, 
    out_start: &mut DateTime<Utc>, out_stop: &mut DateTime<Utc>,) -> Result<bool, HttpServiceError> 
 {
    // Check time format
    if in_message.epoch_format != "UTCGregorian" &&
       in_message.epoch_format != "UTCModJulian" {
        let tmp_msg = format!("ERROR: Invalid epoch format: {}", in_message.epoch_format );
            
        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
    }

    // Check start and end time 
    let rfc3339 = match DateTime::parse_from_rfc3339(in_message.start_time.as_str()) {
        Ok(t) => t,
        Err(e) => {
            let tmp_msg = format!("ERROR: Unable to parse start time: {}", e.to_string());

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
        },
    };

    // Convert to UTC
    *out_start = rfc3339.with_timezone(&Utc);


    let rfc3339 = match DateTime::parse_from_rfc3339(in_message.stop_time.as_str()) {
        Ok(t) => t,
        Err(e) => {
            let tmp_msg = format!("ERROR: Unable to parse stop time: {}", e.to_string());

            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
        },
    };

    // Convert to UTC
    *out_stop = rfc3339.with_timezone(&Utc);

    // Check if the dates are in reverse order
    if out_stop < out_start {
        // Error
        let tmp_msg = format!("ERROR: Stop time is in the past. Smaller thant the start time");

        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
    }

    // Check if they are not separated for more than 1 year
    let interval_duration = out_stop.signed_duration_since(*out_start);

    if interval_duration > chrono::Duration::days(365) {
        // Error to long
        let tmp_msg = format!("ERROR: The propagation period is to big; greater than 1 year");

        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
    }

    if in_message.output.reference_frame != "EarthMJ2000Eq" {
        let tmp_msg = format!("ERROR: Invalid output reference frame: {}", in_message.output.reference_frame.as_str() );

        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
    }

    if in_message.output.output_format != "json" && in_message.output.output_format != "JSON" &&
    in_message.output.output_format != "ccsds-oem" && in_message.output.output_format != "CCSDS-OEM"    {
        let tmp_msg = format!("ERROR: Invalid output ephemeris format: {}", in_message.output.output_format.as_str() );

        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg_id, tmp_msg));
    }

    Ok(false)
}

/**
 * Return index.html
 */
// async fn index(req: HttpRequest) -> actix_web::Result<fs::NamedFile> {
async fn index(in_request: HttpRequest) -> actix_web::Result<actixfs::NamedFile>  { // -> impl Responder
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
async fn not_allowed_method(in_payload: String, in_request: HttpRequest) -> HttpResponse 
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

        "SGP4_SIMPLE" |
        "orb_propagation_sgp4_simple" => {
            usage_msg = fs::read_to_string("doc/orb_propagation_sgp4_simple.html").expect("Unable to read 'doc/orb_propagation_sgp4_simple.html' file");
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
    in_api_version: web::Path<String>,
    in_cfg: web::Data<ConfigVariables>) -> Result<HttpResponse, HttpServiceError>
{
    debug!("Orbit Propagation - SGP4 - TLE Input msg: {}", in_msg.to_string());

    // Check minimum set of fields
    if let Err(e) = check_parameters(&in_msg) {
        return Err(e);
    } 

    if in_api_version != "v1" {
        let tmp_msg = format!("ERROR: Incorrect API version: {}. Only v1 is supported", in_api_version);
            
        error!("{}", tmp_msg.as_str() );
        return Err(HttpServiceError::BadRequest(in_msg.msg_id.clone(), tmp_msg));
    }
    
    // Decode JSON
    let json_message = serde_json::from_value( in_msg.parameters.clone() );
    let orb_propagation_tle_message : OrbPropagationTleStruct = match json_message {
        Ok(msg) => msg,  
        Err(e) => {
            let tmp_msg = format!("ERROR: Unable to decode JSON DeregisterStruct: {}", e.to_string());
            
            error!("{}", tmp_msg.as_str() );
            return Err(HttpServiceError::InternalServerError(in_msg.msg_id.clone(), tmp_msg));
        },
    };

    // Check the specific parameters of the operation
    let mut tle_start_time: DateTime<Utc> = Utc::now();
    let mut tle_stop_time: DateTime<Utc> = Utc::now();

    if let Err(e) = check_operation_parameters(&orb_propagation_tle_message, in_msg.msg_id.clone(),
        &mut tle_start_time, &mut tle_stop_time) {
        return Err(e);
    } 

    debug!("Start time: {}  Stop time: {}", tle_start_time, tle_stop_time);

    let elements = sgp4::Elements::from_tle(
        orb_propagation_tle_message.input.tle.name.clone(),
        orb_propagation_tle_message.input.tle.line1.as_bytes(),
        orb_propagation_tle_message.input.tle.line2.as_bytes(),
    ).map_err(|e| 
        HttpServiceError::InternalServerError(in_msg.msg_id.clone(), e.to_string()) 
    );

    let elements = match elements {
        Ok(el) => el,
        Err(e) => return Err(e),
    };


    let constants = sgp4::Constants::from_elements(&elements)
        .map_err(|e| 
            HttpServiceError::InternalServerError(in_msg.msg_id.clone(), e.to_string())
        );

    let constants = match constants {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    
    // Create output structure
    let mut output_data : OrbPropagationTleResponseStruct = OrbPropagationTleResponseStruct { 
        mission_id:       orb_propagation_tle_message.mission_id, 
        satellite_id:     orb_propagation_tle_message.satellite_id, 
        reference_frame:  orb_propagation_tle_message.output.reference_frame, 
        epoch_format:     orb_propagation_tle_message.epoch_format,  
        ephemeris:        Vec::new(),
    };
    
    let mut satellite_state_vector : SatelliteStateVector = SatelliteStateVector 
    {
        time:             Utc::now().to_rfc3339(),
        position:         [0.0, 0.0, 0.0],
        velocity:         [0.0, 0.0, 0.0],
    };
    
    // Propagate from start time to stop time
    let start_in_min = tle_start_time.timestamp() as f64 / 60.0 as f64;
    let stop_in_min = tle_stop_time.timestamp() as f64 / 60.0 as f64;

    println!(" start time: {}   stop time: {}", start_in_min, stop_in_min);

    let mut i = start_in_min;
    while i < stop_in_min {
        
        // Number of minutes since epoch
        let prediction = constants.propagate(i)
            .map_err(|e| 
                HttpServiceError::InternalServerError(in_msg.msg_id.clone(), e.to_string())
            );
    
        if let Err(e) = prediction  {
            return Err(e);
        }

        let prediction = prediction.unwrap();

        let tmp_timestamp = DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp((i as i64) * 60, 0), Utc );
        
        satellite_state_vector.time = tmp_timestamp.to_rfc3339();
        
        satellite_state_vector.position[0] = prediction.position[0];
        satellite_state_vector.position[1] = prediction.position[1];
        satellite_state_vector.position[2] = prediction.position[2];
        
        satellite_state_vector.velocity[0] = prediction.velocity[0];
        satellite_state_vector.velocity[1] = prediction.velocity[1];
        satellite_state_vector.velocity[2] = prediction.velocity[2];

        // Add current ephemeris to the output list
        output_data.ephemeris.push(satellite_state_vector.clone());
        
        debug!("t = {} min", i);
        debug!("    r = {:?} km", prediction.position);
        debug!("    ṙ = {:?} km.s⁻¹", prediction.velocity);

// TODO: Use requested step

        // Next minute
        i += 1.0;
    }

    match orb_propagation_tle_message.output.output_format.as_str() {
        "json" | "JSON" => {
        },
        "ccsds-oem" | "CCSDS-OEM" => {
            
        },
        _ => {
            
        }
    }; 
    
    let output = RestResponse::new_value(String::from("orb_propagation_sgp4_simple_response"), in_msg.msg_id.clone(), 
    json!(output_data));

    Ok( HttpResponse::Ok().content_type("application/json")
                          .json(output) )
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
    fs::write("orb_tle_propagation.pid", data).expect("Unable to write 'tools.pid' file");


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


    // Data shared between all threads
    {
        let mut tmp_data = GLOBAL_DATA.write().unwrap();

        tmp_data.service_status    = EnumStatus::RUNNING;
    }

    debug!("Creating global data");


    let (tx, rx) = mpsc::channel::<()>();

    let srv = HttpServer::new(move || {
        App::new()

        // limit the maximum amount of data that server will accept
        .data(web::JsonConfig::default().limit( common::common::MAX_SIZE_JSON ))

        .data( config_variables.clone() )

        // Stopping the server
        .data( tx.clone() )

        .service(
            web::scope("/fdsaas")
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
                .route("/{version}/orb_propagation_sgp4_simple", web::get().to(orb_propagation_tle))
                .route("/{version}/OP/SGP4_SIMPLE", web::get().to(orb_propagation_tle))
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
