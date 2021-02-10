/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service 
 * Nanomsg FDS server
 */

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

// Nanomsg
use nng::*;





fn reply() -> Result<()> {
    // Set up the server and listen for connections on the specified address.
    let server = Socket::new(Protocol::Rep0)?;
    server.listen(ADDRESS)?;

    // Receive the message from the client.
    let mut msg = server.recv()?;
    assert_eq!(&msg[..], b"Ferris");

    // Reuse the message to be more efficient.
    msg.push_front(b"Hello, ")?;

    server.send(msg)?;
    Ok(())
}

fn process_message(in_json_message: serde_json::Value, in_server_socket: Socket) -> Result<()> {
    // Create a separated thread and process the message
    let handle = thread::spawn(|| {
        let authorization_key = in_json_message["key"].clone();

        // Check Authorization
        check_authorization(authorization_key);
    
        // Process the message based on its code
        process_a_message(in_json_message, in_server_socket);
    });

    let final_error = handle.join();
    
    if let Err(e) = final_error {
        error!("Error processing the message: {} {}", e.to_string(), e);
    }

    Ok(())
}

fn check_authorization(in_key: String) -> Result<(), error::Error> {
    if in_key == None || in_key.is_empty() == true || in_key != "00998844" {
        debug!("Invalid key: {}", in_key.as_str());
        Err("Invalid key".into())
    } else {
        Ok(())
    }
}

fn process_a_message(in_json_message: serde_json::Value, in_server_socket: Socket) -> Result<()> {
    let tmp_code_id = &in_json_message["msg_code_id"];
    debug!("Received message code: {}", tmp_code_id.as_str())

    match tmp_code_id {

    };

    Ok(())
}

fn usage() {
    println!("Incomplete Worlds (c) 2020");
    println!("Flight Dynamics System (FDS) as a Service");
    println!("");
    println!("Usage:    fds_server   config_file_name");
    println!("");
}

// ================================================================
// *
// *  M  A  I  N
// *
// ================================================================
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 2 {
        usage();
        return Ok(());
    }

    // Init Log
    let error_code = config_log();
    if let Err(e) = error_code {
        println!("ERROR: Unable to read the Log Configuration file: config/log4rs.yaml");
        error!("{} {}", e.to_string(), e);
        return Ok(());
    }

    let now: DateTime<Utc> = Utc::now();

    info!("**********************************");
    info!("Initializing FDS Server: {}", now.to_rfc2822());
    info!("**********************************");

    // Load configuration file
    debug!("Reading the configuration file");

    let tmp_config_filename = args[1].clone();
    let config_variables = read_config(tmp_config_filename);
    let config_variables = match config_variables {
        // Just return the variables
        Ok(tmp_variables) => tmp_variables,
        Err(tmp_error) => {
            error!("Unable to read the configuration file: {}", args[1]);
            error!("Error: {}", tmp_error);
            return Ok(());
        }
    };

    // Connect to the FDS server via Nanomsg 
    let nng_server_address = config_variables["fds_nng_req_address"].clone();
    let nng_server_port    = config_variables["fds_nng_req_port"].clone();
    let nng_ip_address     = nng_server_address + &String::from(":") + &nng_server_port;

    info!("Nanomsg IP Address: {}", nng_ip_address);


    // Create Socket
    let server_socket = Socket::new(Protocol::Rep0)?;
    info!("Socket to FDS server correctly created ");
    
    server_socket.listen( nng_ip_address.as_str() )?;
    info!("Correctly connected to FDS server");


    loop {
        // Read a message
        let input_msg = server.recv()?;
        debug!("Received message: {}", input_msg);

        if let Err(e) = input_msg {
            error!("Error when receiving a message: {}", e.to_string());
            continue;
        }

        let json_buffer = tmp_message.as_slice().to_string();
        let json_message = serde_json::from_str(json_buffer.as_str());

        // If end of processing, then break the loop
        if json_message["operation_id"] == "end" {
            info!("Shutting down the FDS server");
            break;
        }
                
        // Process message in a separated thread
        process_message(json_message, server_socket);
    }

    // Shutting down the server
    server_socket.close();

    now = Utc::now();
    info!("**********************************");
    info!("Finishing FDS Server: {}", now.to_rfc2822());
    info!("**********************************");
}
