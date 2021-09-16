//#![deny(warnings)]
//#![deny(unused_imports)]

/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 */


use std::env;
use std::io;
use std::io::Read;
use std::fs::File;
use std::error;

// Configuration
use std::collections::BTreeMap;

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_yaml;
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
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
// use actix_identity::{CookieIdentityPolicy, IdentityService};

// Redirect to another HTTP
mod redirect;



// limit the maximum amount of data that server will accept
const MAX_SIZE_JSON : usize =  262_144;

const GSAAS_VERSION : &str = "0.1";

// To separate into a dedicated file
#[derive(Serialize, Deserialize, Debug)]
struct LoginMessage {
    username: String,
    hash_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginMessageResponse {
    // TODO: Encoded user id or JWT
    // TODO: Implement authentication
    user_id: String,
}

// struct LogoutMessage {
//     username: String,
//     hashPassword: String,
// }

// struct RegisterMessage {
//    
//}

// struct AppState {
//     jwt: String,
// }





// It propagates the error up (to the caller)
fn config_log(in_filename: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("DEBUG: file name: {}", in_filename);

    if in_filename.is_empty() == true {
        return Err( "Log file name is empty".into() );
    }

    // If we want to modify then handler programmatically
    // let handle = log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    // this line produces a panic!, if it fails
    //log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    // Return any error and convert to std::error::Error;
    log4rs::init_file(in_filename.as_str(), Default::default())?;

    println!("Log Configuration correctly loaded");
    return Ok(());
}

/**
 * Read the GS as a Service configuration file. 
 * It does contain the IP addresses and port of the other modules
 */
fn read_config(config_file_name: String) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    debug!("file name: {}", config_file_name);

    // Open the file and return it. If an error, return the error
    let config_file = File::open( config_file_name.to_string() )?;
    debug!("config file read: ");

    let output_variables = serde_yaml::from_reader( config_file )?;
    debug!("de-serialized: {:?}", output_variables);

    return Ok(output_variables);
}

// Main web page
/**
 * Read the WebContent index.html page and return it
 */
// NOT User
// async fn index() -> impl Responder {
//     debug!("   *** Index");
//     return HttpResponse::Ok().body("Hello world!");

    
// }

// Register
/**
 * Extract the payload data. It should contain:
 * Parameters:
 * - User name
 * - Account user name
 * - Password
 * 
 * Add the user to the database. Separated service???
 * It will return OK, if nothing fails
 });
 */
async fn register() -> impl Responder {
    debug!("   *** Register");

    return HttpResponse::Ok().body("Register");
}

// Login
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
async fn login(info: web::Json<LoginMessage>) -> impl Responder {
    debug!("   *** Login");

    // Extract parametesr

    // Check user
    // Create JWT

    // Reply
    let login_response = LoginMessageResponse{ user_id : String::from("0A4523F6") };   
    return HttpResponse::Ok().json( login_response );
}

// Logout
/**
 * It will receive the JWT and release it. So, user will not be longer logged in
 * Parameters:
 * - username
 * - JWT
 * 
 */
async fn logout() -> impl Responder {
    debug!("   *** Logout");
    return HttpResponse::Ok().body("Logout");
}



// Request to the FDS. It will be forwarded to the FDS server
/**
 * All HTTP requests will be forwarded to the FDS microservice
 */
// async fn forward_fds() -> impl Responder {
//     debug!("   *** FDS API");
//     //return HttpResponse::Ok().body("FDS");

//     //let server_address = config_variables["fdsaas_server_address"].clone();
//     //let server_port    = config_variables["fdsaas_port"].clone();

//     return HttpResponse::Ok();
// }

// Request to the FDS. It will be forwarded to the FDS server
/**
 * All HTTP requests will be forwarded to the MCS microservice
 */
// async fn forward_mcs() -> impl Responder {
//     debug!("   *** MCS");
//     return HttpResponse::Ok().body("MCS API");
// }

/**
 * Forward the request to another HTTP server
 */
// async fn forward(req: HttpRequest, body: web::Bytes, url: web::Data<Url>, client: web::Data<Client>,
// ) -> Result<HttpResponse, Error> {
    
// }

fn usage() {
    println!("Incomplete Worlds (c) 2020");
    println!("Ground Segment as a Service");
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

    let now: DateTime<Utc> = Utc::now(); 

    info!("**********************************");
    info!("Initializing GS as a Service: {}", now.to_rfc2822());
    info!("**********************************");

    // Load configuration file
    debug!("Reading the configuration file");
                
    let tmp_config_filename = args[1].clone();
    let config_variables = read_config(tmp_config_filename.clone());
    let config_variables = match config_variables {
        // Just return the variables
        Ok(tmpVariables) => tmpVariables,
        Err(tmpError) => {
            println!("Unable to read the configuration file: {}", tmp_config_filename.as_str());
            println!("Error: {}", tmpError);
            return Ok(());
        }
    };

    // Init Log
    let tmp_log_filename = config_variables["log_filename"].clone();

    let error_code = config_log(tmp_log_filename.clone());
    if let Err(e) = error_code {
        println!("ERROR: Unable to read the Log Configuration file: {}", tmp_log_filename.as_str());
        println!("{} {}", e.to_string(), e);
        return Ok(());
    }

    let server_address = config_variables["gsaas_server_address"].clone();
    let server_port    = config_variables["gsaas_port"].clone();
    let ip_address     = server_address + &String::from(":") + &server_port;

    info!("Listening IP Address: {}", ip_address);


    // Main loop. Create HTTP server
    // Start http server
    HttpServer::new(move || {
        App::new()
        // limit the maximum amount of data that server will accept
        .data(web::JsonConfig::default().limit( MAX_SIZE_JSON ))

        .service(
            web::scope("/gsaas")
                // Root URL
                // Read the WebContent index.html page and return it
                .service(fs::Files::new("/", "WebContent").index_file("index.html"))

                
                // General
                .service(
                    web::resource("/api/register")
                        //.wrap(redirect::CheckAuthorization),
                        .route(web::post().to(register)),
                )
                .service(
                    web::resource("/api/login")
                        //.wrap(redirect::CheckAuthorization),
                        .route(web::get().to(login)),
                )
                .service(
                    web::resource("/api/logout")
                        //.wrap(redirect::CheckAuthorization),
                        .route(web::get().to(logout)),
                )

                // FDS
                // Authorization will be done in the FDS server
                .service(
                    web::resource("/fdsaas/api")
                        // Redirect to FDS
                        .wrap(redirect::RedirectRequest)
                        // .route(web::get().to(forward_fds))
                        // .route(web::post().to(forward_fds))
                        // .route(web::put().to(forward_fds))
                        // .route(web::delete().to(forward_fds))
                )

                // MCS
                // Authorization will be done in the MCS server
                .service(
                    web::resource("/mcsaas/api")
                        // Redirect to MCS
                        .wrap(redirect::RedirectRequest)
                        // .route(web::get().to(forward_mcs))
                        // .route(web::post().to(forward_mcs))
                        // .route(web::put().to(forward_mcs))
                        // .route(web::delete().to(forward_mcs))
                )
        )
        .service(
            // Authorization will be done in the FDS server
            web::scope("/fdsaas")
                // Root URL
                .service(
                    web::resource("/api")
                    // Redirect to FDS
                    .wrap(redirect::RedirectRequest)
                    // .route(web::get().to(forward_fds))
                    // .route(web::post().to(forward_fds))
                    // .route(web::put().to(forward_fds))
                    // .route(web::delete().to(forward_fds))
                )
        )
    })
    .bind(ip_address)?
    .run()
    .await
}


    



