/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Common functions
 */


// Configuration
use std::fs::File;
use std::collections::BTreeMap;

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct LayerConfig {
    pub index:    u8,
    pub name:     String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigVariables {
    documentation:                  String, 

    pub version:                    String,

    pub config_log_filename:        String,

    pub gsaas_http_address:         String,

    pub decoder_nng_push_address:   String,
    
    pub gs_input_address:           String,
   
    pub gs_output_address:          String,    

    // fixed, fixed_marker, variable
    pub type_of_frame:              String, 
    pub fixed_length:               usize,
    // Hex as String
    pub marker_start:               String,
    // Hex as String
    pub marker_end:                 String,
    // Input and output buffer
    pub max_buffer_size:            usize,

    pub layers:                     Vec<LayerConfig>,
}

impl ToString for ConfigVariables {
    fn to_string(&self) -> String {
        let mut output_buffer : String = format!("Config variables: ");

        output_buffer.push_str( format!("Version: {}\n", self.version).as_str() );
        output_buffer.push_str( format!("Config log file name: {}\n", self.config_log_filename).as_str() );
        output_buffer.push_str( format!("G/Sn Input Address: {}\n", self.gs_input_address).as_str() );
        output_buffer.push_str( format!("G/Sn Output Address: {}\n", self.gs_output_address).as_str() );
        output_buffer.push_str( format!("Type of frame: {}\n", self.type_of_frame).as_str() );

        return output_buffer;
    }
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


