/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * It will implement the entry point and the REST API
 */

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::HashMap;

use tokio::net::TcpStream;
//use tokio::net::UdpSocket;
use tokio::prelude::*;

// Date & Time
use chrono::{DateTime, Utc};

// Log 
use log::{debug, error, info, trace, warn};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};


// Other modules
// ---------------------------------------------

// Common functions
mod common;
use common::{ read_config_json, config_log, ConfigVariables };

mod layers;
use layers::*;

mod cortex_layer;
use cortex_layer::*;

mod nis_layer;
use nis_layer::*;

mod frames;
use frames::*;





async fn read_frame(in_socket: &mut TcpStream, in_config_variables: &ConfigVariables, out_frame: &mut ReceivedFrame) -> Result<usize, String>
{
    let mut n: usize = 0;
    
    match in_config_variables.type_of_frame.as_str() {
        "fixed" => {            
            let mut input_buffer : Vec<u8> = Vec::new(); 
            input_buffer.resize(in_config_variables.max_buffer_size, 0);

            //let mut tmp_buffer : String = String::from("");
            //tmp_buffer.reserve(in_config_variables.fixed_length as usize);
            debug!("Reading fix frame. Length = {}", in_config_variables.fixed_length);

            n = match in_socket.read_exact(&mut input_buffer ).await {
                // socket closed
                Ok(n) if n == 0 => {
                    let msg = String::from("Socket is closed. Finishing");
                    error!("{}", msg);
                    return Err(msg)
                },
                Ok(n) => n,
                Err(e) => {
                    let msg = format!("Error reading from socket; err = {:?}", e);
                    error!("{}", msg);
                    return Err(msg);
                }
            };

            // Check the marker is at the beginning of the frame
            if in_config_variables.type_of_frame.as_str() == "fixed_marker" {
                // Calculate market len in order to only compare that length
                let marker_len = in_config_variables.marker_start.len();

                // Extract just the marker
                let tmp_string = String::from_utf8( input_buffer[0..marker_len].to_vec() ).unwrap();

                if tmp_string.as_str().starts_with(in_config_variables.marker_start.as_str()) == false {
                    let msg = format!("Fix frame does not start with the marker: {}", in_config_variables.marker_start);

                    error!("{}", msg);
                    return Err(msg);
                }
            }

            out_frame.data = input_buffer;
        },

        "variable" => {
            info!("Not implemented yet");
        },

        _ => {
            error!("Unknown frame type");
            return Err(String::from("Unknow frame type"));
        },
    };

    out_frame.header.creation_time = Utc::now().to_rfc3339();

    Ok(n)
}


/**
 * 
 */
fn create_layers(in_config_variables: &ConfigVariables, in_list_layers: &mut HashMap<u8, Box<dyn LayerTrait>>)
{
    for l in &in_config_variables.layers {
        let mut new_layer : Box<dyn LayerTrait>;

        // Create the Layer object
        match l.name.as_str() {
            "cortex_layer" => {
                new_layer = Box::new(
                    CortexLayer::new(true)
                );
            },
            
            "nis_layer" => {
                new_layer = Box::new(
                    NisLayer::new(true)
                );
            },
            
            "ccsds_cltu_layer" => {
                new_layer = Box::new(
                    CortexLayer::new(true)
                );                
            },

            _ => {
                error!("Unknown layer type: {}   IGNORED", l.name);
                continue;
            }
            
        }

        // Load the configuration
        new_layer.load_configuration(&in_config_variables);
        
        // Add to the hash map
        // 0 - is the root layer
        in_list_layers.insert(l.index, new_layer);
    }
}

/**
 * If the field is a CRC, check it
 */
fn check_crc(in_field: &FrameFieldValue) -> Result<u32, String>
{
    Ok(0)
}

/**
 * Check the field is whithin expected limits
 */
fn check_ool(in_field: &FrameFieldValue)
{

}

/**
 * Check the values of the field
 */
fn check_values(in_field: &FrameFieldValue)
{

}

/**
 * Convert from raw to engineering value
 */
fn calibrate(in_field: &FrameFieldValue, out_field: &mut FrameFieldValue)
{

}

/**
 * Process the raw frame and extract the processed frames
 * Apply all layers of protocols
 * 
 */
fn process_frame(in_config_variables: &ConfigVariables, in_frame: &ReceivedFrame, 
                 in_list_layers : &mut HashMap<u8, Box<dyn LayerTrait> >, out_json_string: &mut String) -> Result<u32, String>
{
    // First chunk of data to be processed
    let mut data = in_frame.data.clone();

    let mut output_frame: ProcessedFrame = ProcessedFrame::new();

    for i in 0 .. in_list_layers.len() as u8 {
        let layer = match in_list_layers.get_mut(&i) {
            Some(l) => l,
            None => {
                let msg = format!("Undefined layer found. Index: {}", i);
                error!("{}", msg);
                return Err(msg);
            },
        };

        let mut layer_fields: ProcessedLayerFrame = ProcessedLayerFrame::new();

        layer_fields.name = layer.get_name().clone();

        // if let Err(e) = layer.process(in_frame.data.as_slice()) {
        //     let msg = format!("Error processing frame: {}", e);
        //     error!("{}", msg);
        //     return Err(msg);
        // }
        if let Err(e) = layer.receive(data.as_slice(), &mut layer_fields) {
            let msg = format!("Error processing input frame: {} Index: {}", e, i);
            error!("{}", msg);
            return Err(msg);
        }

        // Process all fields
        let mut new_layer_data: Vec<FrameFieldValue> = Vec::new();

        for f in layer_fields.layer_data.iter_mut() {
            debug!("Field: {}", f.name);
            
            if f.pdu_flag == true {
                // Data to be processed by the next level
                data.clear();
                
                for b in f.value.as_bytes() {
                    data.push(*b);
                }
            }
            
            if let Err(e) = check_crc(&f) {
                let msg = format!("Error processing CRC field: {} Index: {}", e, i);
                error!("{}", msg);
                break;
            }
            
            // Check Out of Limits
            check_ool(f);
            
            // Check values
            check_values(f);
            
            // Calibrate raw values. It does required a S/C Database
            let mut tmp_new_field: FrameFieldValue = FrameFieldValue::new();

            calibrate(f, &mut tmp_new_field);
            
            // Add to layer. Move it
            new_layer_data.push(tmp_new_field);
        }

        layer_fields.layer_data.clear();
        layer_fields.layer_data.append(&mut new_layer_data);

        // Add output
        output_frame.data.push(layer_fields);
    }

    // Generate output
    *out_json_string = match serde_json::to_string(&output_frame) {
        Ok(o) => o,
        Err(e) => {
            let msg = format!("Error generating output JSON frame: {}", e);
            error!("{}", msg);
            return Err(msg);
        },
    };

    Ok(0)
}

fn usage() {
    println!("Incomplete Worlds (c) 2020");
    println!("Ground Segment (GS) as a Service");
    println!("");
    println!("Usage:    main   config_file_name");
    println!("");
}

// ================================================================
// *
// *  M  A  I  N
// *
// ================================================================
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 2 {
        usage();
        return;
    }
    
    let now: DateTime<Utc> = Utc::now(); 

    println!("**********************************");
    println!("Initializing GS as a Service - TM: {}", now.to_rfc3339() );
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
            return;
        }
    };

    // Init Log
    let tmp_log_filename = config_variables.config_log_filename.clone();

    let error_code = config_log(&tmp_log_filename);
    if let Err(e) = error_code {
        println!("ERROR: Unable to read the Log Configuration file: {}", tmp_log_filename.as_str());
        println!("{} {}", e.to_string(), e);
        return;
    }

    info!("**********************************");
    info!("Initializing GS as a Service - TM: {}", now.to_rfc3339() );
    info!("**********************************");

    // External HTTP Address. It will listen for HTTP requests coming from this address
    let input_address = config_variables.gs_input_address.clone();
    let output_address = config_variables.gs_output_address.clone();
   
    info!("Listening IP Address: {}", input_address);
    info!("Sending data IP Address: {}", output_address);



    // Open the connections
    // 1 - Input read from G/Sn
    let mut input_stream;

    if input_address.contains("tcp") == true || input_address.contains("TCP") == true {
        input_stream = TcpStream::connect( input_address ).await.unwrap();

    } 
    // else if input_address.contains("udp") == true || input_address.contains("UDP") == true {
    //     // let socket = match input_address.parse()
    //     //     .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e)).unwrap()
    //     // {
    //     //     SocketAddr::V4(a) => UdpBuilder::new_v4()?.bind(a),
    //     //     SocketAddr::V6(a) => UdpBuilder::new_v6()?.only_v6(true)?.bind(a),
    //     // };
        
    //     // input_stream = UdpSocket::from_std(socket).await.unwrap();
        
    //     let socket_addr = SocketAddr::new( IpAddr::V4( Ipv4Addr::new(127, 0, 0, 1)), 8080);

    //     //input_stream = UdpSocket::connect( input_address ).await.unwrap();
    //     //input_stream = UdpSocket::bind(input_address).await.unwrap();

    //     input_stream = UdpSocket::bind(("0.0.0.0", 44667)).await.unwrap();

 
    /*

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        let buf = &mut buf[..amt];
        buf.reverse();
        socket.send_to(buf, &src)?;
        */
    //}
    else {
         error!("Invalid input address: {}", input_address);
        return;
    }
    info!("Input stream created");


    // 2 - Output of JSON
    let mut output_stream;

    if output_address.contains("tcp") == true || output_address.contains("TCP") == true {
        output_stream = TcpStream::connect( output_address ).await.unwrap();

    } else {
        error!("Invalid output address: {}", output_address);
       return;
    }
    info!("Output stream created");

    // Create the stack of layers
    let mut list_layers : HashMap<u8, Box<dyn LayerTrait> > = HashMap::new();

    create_layers(&config_variables, &mut list_layers);

    // Read a frame
    let mut new_frame : ReceivedFrame = ReceivedFrame {
        header:  BaseFrame {
            id:             0,
            creation_time:  Utc::now().to_rfc3339(),
        },
        data : Vec::new(),
    };

    let mut output_json : String;

    // While true
    loop {

        if let Err(e) = read_frame(&mut input_stream, &config_variables, &mut new_frame).await {
            error!("{}", e);
            continue;
        }
        
        // Decode frame by applying layers
        output_json = String::from("");

        if let Err(e) = process_frame(&config_variables, &new_frame, &mut list_layers, &mut output_json) {
                error!("Error create JSON output object");
                continue;
        };
        
        // Send JSON to output connection
        //if let Err(e) = output_stream.write_all(&buf[0..n]).await {
        if let Err(e) = output_stream.write_all(output_json.as_bytes()).await {
            error!("Failed to write to socket; err = {:?}", e);
            return;
        }
    }

    // let result = input_stream.write(b"hello world\n").await;
    // println!("wrote to stream; success={:?}", result.is_ok());


    // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6142);
    // let mut stream = TcpStream::connect(addr).await.unwrap();

    // let result = stream.write(b"hello world\n").await;

}