/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Common functions
 */


// Configuration
use std::fs::File;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::{fmt::Write, num::ParseIntError};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json;

#[macro_use]
use lazy_static::lazy_static;



#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigVariables {
    documentation:                  String, 

    pub version:                    String,

    pub config_log_filename:        String,
    
    // JWT Secret key
    pub secret_key:                 String,
    
    pub tools_http_address:         String,
}


impl ConfigVariables 
{
    pub fn new() -> Self 
    {
        ConfigVariables {
            documentation:              String::new(),
            version:                    String::new(),
            config_log_filename:        String::new(),
            secret_key:                 String::new(),
            tools_http_address:         String::new(),
        }
    }
}

impl ToString for ConfigVariables {
    fn to_string(&self) -> String {
        let mut output_buffer : String = format!("Config variables: ");

        output_buffer.push_str( format!("Version: {}\n", self.version).as_str() );
        output_buffer.push_str( format!("Config log file name: {}\n", self.config_log_filename).as_str() );
        output_buffer.push_str( format!("Tools HTTP Address: {}\n", self.tools_http_address).as_str() );

        return output_buffer;
    }
}

/**
 * Global configuration of the service
 * It also signals when to stop all lops
 */
lazy_static! {
    pub static ref CONFIG_VARIABLES : Arc< RwLock< ConfigVariables >>  = Arc::new( RwLock::new( ConfigVariables::new() ) );
}
//
// ====================================================================
// ====================================================================
//

// It propagates the error up (to the caller)
pub fn config_log(in_filename: &String) -> Result<(), Box<dyn std::error::Error>> {
    println!("DEBUG: file name: {}", in_filename);

    if in_filename.is_empty() == true {
        return Err("Log file name is empty".into());
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
 * 
 */
pub fn read_config_yaml(config_file_name: &String) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    println!("DEBUG: file name: {}", config_file_name);

    if config_file_name.is_empty() == true {
        return Err("Config file name is empty".into());
    }

    // Open the file and return it. If an error, return the error
    let config_file = File::open(config_file_name.to_string())?;
    println!("DEBUG: config file read: ");

    let output_variables = serde_yaml::from_reader(config_file)?;
    println!("DEBUG: de-serialized: {:?}", output_variables);

    return Ok(output_variables);
}

pub fn read_config_json(config_file_name: &String) -> Result<ConfigVariables, Box<dyn std::error::Error>> {
    println!("DEBUG: file name: {}", config_file_name);

    if config_file_name.is_empty() == true {
        return Err("Config file name is empty".into());
    }

    // Open the file and return it. If an error, return the error
    let config_file = File::open(config_file_name.to_string())?;
    println!("DEBUG: config file read: ");

    let output_variables : ConfigVariables = serde_json::from_reader(config_file)?;
    println!("DEBUG: de-serialized: {}", output_variables.to_string());

    return Ok(output_variables);
}


