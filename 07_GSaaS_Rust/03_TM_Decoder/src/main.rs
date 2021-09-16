/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service
 * TM Decoder main
 * 
 * Receive a message via the REQ/REP socket
 * 
 * Msg: Configure
 * Configure the decoder. Configuration will include:
 *  List of layers and their parameters
 *  Source IP and port
 *  Target IP and port
 *  Output stream type; NanoMsg, ZeroMsg (???), HTTP+REST
 *  Output format; CSV, JSON
 *  Output value type; bin, dec, hex
 *  Little endian flag
 * 
 * Every time a message of this type is received, config will be overwritten
 * 
 * Msg: Connect
 *  Create new thread
 *  Open connection to source IP
 *  Open connection to target IP, depending on stream type
 *   
 *  While no stop:
 *    Wait for frame
 *    Process frame accordantly to the layers and their configuration
 *    Generate output message accordantly to output format and value types
 *    Send output message to Target
 * 
 * Msg: Disconnect
 *  Wait for current message to be sent to Target and stop processing messages
 *  Close connection to source IP
 *  Close connection to target IP
 *  End async task
 * 
 * Msg: Shutdown
 *  Stop the decoder
 */

use std::env;
use std::result::Result;


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

// New Nanomsg
use nng::options::{Options};
use nng::{Aio, AioResult, Context, Message, Protocol, Socket, Error};



// Common functions
mod common_tm_decoder;
use common_tm_decoder::{ read_config_json, config_log, ConfigVariables };

mod api_messages;
use api_messages::*;



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


/**
 * It does receives messages from the HTTP thread or the NNG thread
 * After processing them, send the reply back to the caller
 * 
 */
fn main_control_loop(in_config_variables: ConfigVariables) {
    info!("**** Starting MAIN control loop ");

    // Create REP Socket
    // ---------------------------------------------------
    let rep_control_socket = Socket::new(Protocol::Rep0);
    let rep_control_socket   = match rep_control_socket  {
        Ok(s) =>  { info!("REP Socket from TM Decoder correctly created ");
                    s
        },
        Err(e) => {
            error!("Unable to create REP socket. Error: {}", e.to_string());
            return;
        },
    };

    // Start listening
    let unused_result = rep_control_socket.listen( in_config_variables.nng_server_address.as_str() );

    if let Err(e) = unused_result {
        error!("Error when starting listening the socket: {}", e );
        return;
    }
    info!("Correctly connected to REQ/REP socket. Address: {}", in_config_variables.nng_server_address);


    let mut done_flag = false;
    let mut output_json_message : String = String::new();

    while done_flag == false {
        let input_msg = rep_control_socket.recv();
        //debug!("Received message: {}", input_msg.unwrap().as_slice() );
        
        if let Err(e) = input_msg {
            error!("Error when receiving a message: {}", e );
            // End of the loop
            break;
        }

        // As u8[]
        let json_buffer = input_msg.unwrap();

        debug!("Received MAIN message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );

        // Decode JSON
        let json_message = serde_json::from_slice(json_buffer.as_slice());
        let json_message : Value = match json_message {
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

        // Check minimum set of fields
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

        // Stop FDS server???
        if json_message["msg_code_id"] == String::from("shutdown") {
            // Check magical word
            if json_message["exit_code"] == String::from("XYZZY") {
                done_flag = true;

                info!("*** Main. Leaving");

                // Send back the answer
                output_json_message = build_api_answer_str_json(false, "", "");

                let mut socket_m = nng::Message::from(output_json_message.as_bytes());
                
                rep_control_socket.send(socket_m);
                continue;
            }
        }
        
        // Process message
        let response_message = process_message(&json_message);

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

        // Send back the answer
        let mut socket_m = nng::Message::from(output_json_message.as_bytes());

        rep_control_socket.send(socket_m);
    }

    info!("**** Stopping MAIN control loop ");
}

/**
 * Process all type of messages
 */
fn process_message(in_json_message: &serde_json::Value) -> Result<String, String> 
{
// Create a separated thread and process the message
//let handle = thread::spawn(|| {
    //let authorization_key : String = in_json_message["authentication_key"].to_string();

    // Check Authorization
    //if authorization_manager::check_authorization( &authorization_key ).unwrap() == false {
    //    error!("Non authorized");
    //    Ok( String::from("Not authorized") )
    //}
    

    let tmp_code_id = in_json_message["msg_code_id"].as_str().unwrap();

    debug!("Received message code: {}", tmp_code_id);

    match tmp_code_id {
        "configure" => {
            configure_decoder(&in_json_message)
        },

        "connect" => {
            connect(&in_json_message)
        },
                    
        "disconnect" => {
            disconnect(&in_json_message)
        },

        _ => { println!("Unknown message code: {}", tmp_code_id);
            error!("Unknown message code: {}. IGNORED", tmp_code_id);
            Ok(String::new())
        }
    };

    Ok(String::new())
}

/**
 * Create a new thread and configure it accordantly to the received configuration message
 */
fn configure_decoder(in_json_message: &serde_json::Value) -> Result<String, String> 
{

    // Create the stack of layers
    let mut list_layers : HashMap<u8, Box<dyn LayerTrait> > = HashMap::new();

    create_layers(&config_variables, &mut list_layers);





    Ok(String::new())
}

/** 
 * Connect to the source of frames, usually a ground station
 */
fn connect(in_json_message: &serde_json::Value) -> Result<String, String> 
{
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


    Ok(String::new())
}

/**
 * Disconnect from the source of frames
 * Destroy the thread
 */
fn disconnect(in_json_message: &serde_json::Value) -> Result<String, String> 
{
    Ok(String::new())
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
    println!("TM Decoder");
    println!("");
    println!("Usage:    main   config_file_name  decoder_id   nng_address");
    println!("");
}

// ================================================================
// *
// *  M  A  I  N
// *
// ================================================================
fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Arguments: {:?}", args);

    if args.len() < 4 {
        usage();
        return;
    }
    
    let mut now: DateTime<Utc> = Utc::now(); 

    println!("**********************************");
    println!("Initializing TM Decoder : {}", now.naive_utc() );
    println!("**********************************");

    // Load configuration file
    debug!("Reading the configuration file");
                
    let tmp_config_file_name = args[1].clone();
    let config_variables = read_config_json(&tmp_config_file_name);
    let mut config_variables = match config_variables {
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
    info!("Initializing TM Decoder: {}", now.naive_utc() );
    info!("**********************************");

    config_variables.decoder_id = args[2].clone();
    config_variables.nng_server_address = args[3].clone();
   
    info!("Listening NNG Address: {}", config_variables.nng_server_address);

    main_control_loop(config_variables);
}
