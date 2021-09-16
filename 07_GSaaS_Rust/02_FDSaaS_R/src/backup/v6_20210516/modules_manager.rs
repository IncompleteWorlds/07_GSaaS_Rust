/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Modules Manager
 * It invokes the external modules that processes the messages
 * Keep record of execution
 * Process reply messages and wake up dormant tasks
 */

use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::result::Result;
use std::sync::{Arc, Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::task::{Context, Poll, Waker};
use std::{thread, thread::JoinHandle, time, time::Duration};

// Log
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Date & Time
use chrono::{DateTime, Utc};

// New Nanomsg
use nng::*;

// Wait for a task to be completed
//use crate::wait_for_task::*;

// Messages
use crate::fds_messages::*;

// Task manager
use crate::tasks_manager::*;
use TASK_MANAGER;

// Common functions
use crate::common::*;
use CONFIG_VARIABLES;


// Definition of types
//-------------------------------------------------

#[derive(Serialize, Deserialize)]
enum EnumModuleType {
    INTERNAL,
    EXTERNAL,
}

#[derive(Serialize, Deserialize)]
enum EnumVariableType {
    NONE,
    STRING,
    INTEGER,
    UNSIGNED_INTEGER,
    FLOAT,
    DOUBLE,
    BOOLEAN,
}

// This is need for checking only some values
#[derive(PartialEq)]
enum EnumModuleStatus {
    IDLE,
    RUNNING,
    ERRONEOUS,
    STOPPED,
}

// This is need for checking only some values
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumExecutionStatus {
    IDLE,
    RUNNING,
    RESPONSE_RECEIVED,
    COMPLETED,
    STOPPED,
    CANCELED,
}

#[derive(Serialize, Deserialize)]
struct OutputVariableDefinition {
    definition:      VariableDefinition,
    database:        String,
    db_table_name:   String,
    db_column_name:  String,
}

#[derive(Serialize, Deserialize)]
struct VariableDefinition {
    name:           String,
    description:    String,
    default_value:  String,
    // integer, unsigned, bool, float, double, string
    value_type:     EnumVariableType,
}

#[derive(Serialize, Deserialize)]
struct ModuleDefinition {
    name:              String,
    description:       String,
    module_type:       EnumModuleType,
    binary_file:       String,
    binary_file_path:  String,
    working_directory: String,
    config_file:       String,
    arguments:         String,
    messages:          Vec<String>,
    input_variables:   Vec<VariableDefinition>,
    output_variables:  Vec<OutputVariableDefinition>,
}

/**
 * It describres a Running Module
 * Each module can have several instances running in Parallel
 */
pub struct Module {
    definition:         ModuleDefinition,

    id:                 u32,

    // TODO: Add a list of instances
    //       Add functions for adding or removing instances from that list
    instance_id:        u32,
    push_address:       String,
    push_port:          u32,
    push_socket:        nng::Socket,
    sub_address:        String,
    req_address:        String,
    status:             EnumModuleStatus,
    // UTC time. Format;   yyyy-mm-ddThh:mi:ss.sss
    start_time:         DateTime<Utc>,
    stop_time:          DateTime<Utc>,
    // Process, after fork
    child_process:      Option<Child>,
}

//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
pub struct ExecutionRecord {
    execution_id:        u32,
    user_id:             u32,
    module_id:           u32,
    module_instance_id:  u32,
    start_time:          String,
    stop_time:           String,
    status:              EnumExecutionStatus,
    answer:              String,
    //pair:                  Arc<(Mutex<bool>, Condvar)>,
    //wait_task:             Option<WaitForAnswerFuture>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExecutionRecordModuleAnswer {
    execution_id:        u32,
    user_id:             u32,
    module_id:           u32,
    module_instance_id:  u32,
    answer:              String,
}

/* *
 * Definition of the Module Manager pseudo-class
 */
pub struct InternalModuleData {
    // Private
    // ---------------------
    pub list_running_modules: Vec<Module>,
    //pub list_executions:         Vec<ExecutionRecord>,
    //pub executions_counter:      u32,

    // Public
    // ---------------------
}

impl InternalModuleData {
    // This function will create an instance of the Module Manager
    pub fn new() -> Self {
        InternalModuleData {
            list_running_modules: Vec::new(),
            //list_executions:          Vec::new(),
            //executions_counter:       0,
        }
    }
}

pub struct ModuleManager {
    data: RwLock<InternalModuleData>,
}

thread_local! {
   pub static MODULE_MANAGER : Arc<ModuleManager> = Arc::new( ModuleManager::new() );
//    //static TASK_MANAGER : RwLock< Arc<InternalTaskData> >  = RwLock::new( Arc::new( InternalTaskData::new() ) );
}

impl ModuleManager {
    // This function will create an instance of the Module Manager
    pub fn new() -> ModuleManager {
        ModuleManager {
            data: RwLock::new(InternalModuleData::new()),
        }
    }

    pub fn current() -> Arc<ModuleManager> {
        MODULE_MANAGER.with(|c| c.clone())
    }

    /**
     * Load the list of module definitions from the JSON config file
     * For each module, it runs the executable
     */
    //pub async fn load_module_definitions(&self, in_config_variables: &ConfigVariables) -> std::result::Result<(), String>
    pub fn load_module_definitions(&self) -> std::result::Result<(), String> {
        let tmp_config_data = CONFIG_VARIABLES.read().unwrap();

        let module_file_name = tmp_config_data.modules_definition_file.clone();

        // Open the file and return it. If an error, return the error
        let module_file = File::open(&module_file_name).unwrap();
        debug!(
            "Modules definition file read: {}",
            module_file_name.as_str()
        );

        let the_definitions = serde_json::from_reader(module_file);
        let tmp_list: Vec<ModuleDefinition> = match the_definitions {
            Ok(l) => l,
            Err(e) => {
                let error_msg = format!("Unable to read definitions from: {}", module_file_name);

                error!("{}", error_msg);
                error!("{}", e);

                return Err(error_msg);
            }
        };

        // A random number
        let mut next_id = 20;

        for a_definition in tmp_list {
            let tmp_pull_port = tmp_config_data
                .modules_base_pull_port
                .trim()
                .parse::<u32>()
                .expect("Incorrect PULL base port number");

            let mut new_module = Module {
                definition: a_definition,

                id: next_id,
                instance_id: 1,

                push_address: tmp_config_data.modules_base_pull_address.clone(),
                push_port: (tmp_pull_port + next_id),

                push_socket: nng::Socket::new(nng::Protocol::Push0).unwrap(),

                sub_address: tmp_config_data.fds_nng_sub_address.clone(),

                req_address: tmp_config_data.fds_nng_rep_address.clone(),

                status: EnumModuleStatus::IDLE,

                // UTC time. Format;   yyyy-mm-ddThh:mi:ss.sss
                start_time: Utc::now(),
                stop_time: Utc::now(),

                // Process
                child_process: None,
            };

            info!("   Loading module: {} - {}", new_module.definition.name, new_module.id);

            // Do some checkings
            if new_module.definition.binary_file.is_empty() == true {
                let error_msg = format!("Binary file name is empty");

                error!("{}", error_msg);

                return Err(error_msg);
            }

            // Run the module
            let _tmp_result = self.run_module(&mut new_module);
            if let Err(e) = _tmp_result {
                let error_msg = format!(
                    "Unable to execute module: {} error: {}",
                    new_module.definition.binary_file.as_str(),
                    e
                );

                error!("{}", error_msg);

                //If there is an error, the module will be added to the list anyway but in an erroneous state
            }

            // Add module to the list
            {
                let m = ModuleManager::current();
                let mut tmp_data = m.data.write().unwrap();

                tmp_data.list_running_modules.push(new_module);
            }
            // Increment counter
            next_id += 1;
        }

        Ok(())
    }

    /**
     * Call a module
     * The module will receive a message and execute it. It will return a message either with the output
     * or a link to the file containing the answer
     *
     * Return: A JSON object either describing the answer or a link to the resource containing the answer
     */
    pub fn call_module(&self, in_json_message: &mut InternalMessage) -> Result<InternalResponseMessage, String> {
        info!("Processing message JSON: {}", in_json_message.to_string());

        let mut received_response: InternalResponseMessage = InternalResponseMessage::new();
        let tmp_msg_code_id = in_json_message.request.msg_code_id.clone();
        let tmp_msg_id = in_json_message.request.msg_id.clone();

        // Get user id
        let user_id = in_json_message.user_id.clone();
        //let user_id : String;
        // if in_json_message.request.parameters["user_id"].is_null() == false {
        //     match in_json_message.request.parameters["user_id"].as_str() {
        //     Some(u) => user_id = String::from(u),
        //     None => {
        //         let error_msg = String::from("user_id not found in the JSON message");
        //         error!("{}", error_msg);

        //         return Err( error_msg );
        //     },
        //     };
        // } else {
        //     let error_msg = String::from("user_id not found in the JSON message");
        //     error!("{}", error_msg);

        //     return Err( error_msg );
        // }

        // Look for the module that can execute the message
        let mut module_found = false;

        let tmp_data = self.data.read().unwrap();

        for (i, current_module) in tmp_data.list_running_modules.iter().enumerate() {
            if current_module.definition.messages.contains(&tmp_msg_code_id) == true
            {
                // Check if module is running
                if current_module.status != EnumModuleStatus::RUNNING {
                    info!("Module is not running. It will be started");

                    {
                        let m = ModuleManager::current();
                        let mut tmp_mut_data = m.data.write().unwrap();

                        let tmp_current_module = &mut tmp_mut_data.list_running_modules[i];

                        if let Err(e) = self.run_module(tmp_current_module) {
                            error!("{}", e.to_string());

                            return Err(e.to_string());
                        }
                    }
                }
                debug!("Module: {} is running", current_module.definition.name);

                // Add execution record
                let task_id;
                {
                    task_id = TASK_MANAGER.write().unwrap().add_task(
                        current_module.id,
                        current_module.instance_id,
                        user_id,
                    );
                }

                // Add execution id to the JSON message
                // let mut new_json_message: RestRequest = in_json_message.clone();

                // new_json_message.parameters["execution_id"] = json!(task_id);
                in_json_message.execution_id = task_id;


                // Send messages
                let tmp_message = Message::from(&serde_json::to_vec(&in_json_message).unwrap());

                // Send the message to the main control loop of the module
                // Message is a JSON
                debug!("Sending message: {}", in_json_message.to_string());

                let status = current_module.push_socket.send(tmp_message);
                if let Err(e) = status {
                    let error_msg = format!(
                        "Error when sending JSON message to Module: {} Error: {}",
                        current_module.definition.name,
                        nng::Error::from(e)
                    );
                    error!("{}", error_msg);

                    return Err(error_msg);
                }

                // Create response
                received_response = InternalResponseMessage::new_wait(tmp_msg_code_id.as_str(), 
                                                                      tmp_msg_id, task_id);

                // Return the received response
                module_found = true;
                break;
            }
        }

        if module_found == false {
            let error_msg = format!(
                "Error: Module to process message: {} not found",
                tmp_msg_code_id
            );
            error!("{}", error_msg);

            return Err(error_msg);
        }

        Ok(received_response)
    }

    /**
     * Spawn a process for executing an external module
     * Stores the PID of the module
     * Set the start time of the process

     TODO: Rename to add_instance
     It shall start an instance
    */
    fn run_module(&self, in_module: &mut Module) -> Result<u32, String> {
        debug!("Running module: {}", in_module.definition.name);

        let mut command_line: String = String::from("");

        if in_module.definition.binary_file_path.is_empty() == false {
            command_line.push_str(in_module.definition.binary_file_path.as_str());
            command_line.push_str("/");
        }
        command_line.push_str(in_module.definition.binary_file.as_str());

        let tmp_pull_address: String = in_module.push_address.clone() + &String::from(":") + 
                                       &in_module.push_port.to_string();

        let mut tmp_config_file: String = String::from("config.json");

        if in_module.definition.config_file.is_empty() == false {
            tmp_config_file = in_module.definition.config_file.clone();
        }

        // Command line:
        //   executable  config_file.json   instance_id   pull_address   sub_address   req_address
        let mut child_process = Command::new(command_line.as_str());

        if in_module.definition.working_directory.is_empty() == false {
            child_process.current_dir(in_module.definition.working_directory.clone());
        }

        if in_module.definition.arguments.is_empty() == false {
            child_process.arg(in_module.definition.arguments.clone());
        }

        let child_process = child_process
            .arg(tmp_config_file)
            .arg(in_module.instance_id.to_string())
            .arg(tmp_pull_address)
            .arg(in_module.sub_address.clone())
            .arg(in_module.req_address.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            //.stderr(Stdio::null())
            //.stdout( Stdio::piped() )
            .spawn();

        info!("Executing Module: {} Cli: {} ", in_module.definition.name, command_line);

        let tmp_result = match child_process {
            Ok(c) => c,
            Err(e) => {
                let error_msg = format!("Error while executing: {}", command_line.as_str());

                error!("{}", error_msg);
                error!("{}", e);

                // Set the status
                in_module.status = EnumModuleStatus::ERRONEOUS;

                return Err(error_msg);
            }
        };

        // Write the PID to a file
        let pid_data = format!("{}", tmp_result.id());
        let pid_file_name = in_module.definition.name.clone() + &String::from(".pid");
        debug!("Module PID file: {}", pid_file_name);
        fs::write(pid_file_name, pid_data).expect("Unable to write PID module file");

        // Move the Child
        in_module.child_process = Some(tmp_result);

        // Set the start time as the current time
        //new_module.start_time = format!("{}", now.naive_utc());
        //new_module.stop_time  = format!("{}", now.naive_utc());
        in_module.start_time = Utc::now();
        in_module.stop_time = Utc::now();

        // Small sleep 1 sec
        //thread::sleep(Duration::from_secs(1));


        /*

        // Try to connect to module
        let mut error_flag = false;
        let mut error_msg = String::new();

        let push_socket_address = in_module.push_address.clone() + &String::from(":") + 
                                 &in_module.push_port.to_string();

        for i in 0..3 as u8 {
            // Give the module time to start
            thread::sleep(Duration::from_secs(2));

            info!("   Iteration: {}", i);
            error_flag = false;

            // Create the PUSH socket
            let tmp_push_socket = Socket::new(Protocol::Push0);
            in_module.push_socket = match tmp_push_socket {
                Ok(s) => {
                    info!("PUSH Socket to {} correctly created ", in_module.definition.name);
                    s
                }
                Err(e) => {
                    // // Set the status
                    // in_module.status = EnumModuleStatus::ERRONEOUS;

                    error_msg = format!(
                        "Unable to create PUSH Socket to {}. Error: {}",
                        in_module.definition.name,
                        e.to_string()
                    );
                    error!("{}", error_msg.as_str());

                    // // Kill the module
                    // if in_module.child_process.is_some() {
                    //    error!("Module: {} will be killed", in_module.definition.name);

                    //    // Get exit status and kill the module
                    //    self.kill_module(in_module).await;
                    // }

                    // return Err(error_msg);
                    error_flag = true;

                    // Give the module time to start
                    //tokio::time::delay_for(Duration::from_secs(1)).await;

                    continue;
                }
            };

            // Start listening. It does need the module to be able to connect to. So, it must be up and running
            let unused_result = in_module.push_socket.dial(push_socket_address.as_str());
            if let Err(e) = unused_result {
                // Set the status
                //in_module.status = EnumModuleStatus::ERRONEOUS;

                let error_msg = format!(
                    "Error when connecting to PUSH socket using address: {}\n Error: {}",
                    push_socket_address, e
                );

                error!("{}", error_msg);

                // Kill the module
                // if in_module.child_process.is_some() {
                //    error!("Module: {} will be killed", in_module.definition.name);
                //    // Get exit status and kill the module
                //    self.kill_module(in_module).await;
                // }
                // return Err(error_msg);
                error_flag = true;
                // Give the module time to start
                //tokio::time::delay_for(Duration::from_secs(1)).await;
                continue;
            } else {
                // End of the loop
                break;
            }
        }

        if error_flag == true {
            // Set the status
            in_module.status = EnumModuleStatus::ERRONEOUS;

            //let error_msg = format!("Unable to create PUSH Socket to {}. Error: {}", in_module.definition.name, e.to_string());
            //error!("{}", error_msg.as_str());

            // Kill the module
            if in_module.child_process.is_some() {
                error!("Module: {} will be killed", in_module.definition.name);

                // Get exit status and kill the module
                self.kill_module(in_module);
            }

            return Err(error_msg);
        } else {
            info!("Correctly connected to PUSH socket. Address: {}", push_socket_address);

            // Set the status
            in_module.status = EnumModuleStatus::RUNNING;
        }
        */
        Ok(0)
    }

    /**
     * Handle answer from modules
     * It will receive the answer and wake up the dormant task
     */
    pub fn handle_module_answer(&self, in_json_message: &InternalResponseMessage) -> Result<InternalResponseMessage, String> {
        info!("Processing answer from module JSON: {}", in_json_message.to_string());



        // Deserialize input message
        //   let response_json_message  = serde_json::from_value( in_json_message.clone() );
        //   // let response_json_message : ExecutionRecordModuleAnswer = match response_json_message {
        //   let response_json_message : RestRequest = match response_json_message {
        //         Ok(msg) => msg,
        //         Err(e) => {
        //            let tmp_msg = format!("ERROR: Incorrect SUB answer message. Error: {}", e.to_string());
        //            error!("{}", tmp_msg.as_str() );

        //            return Err(tmp_msg);
        //         },
        //   };
        
        //let tmp_execution_id : u32 = in_json_message.execution_id;
        // let mut tmp_request : RestRequest;

        // //if in_json_message.execution_id == 0 {
        // if in_json_message.parameters["execution_id"].is_null() == true {
        //     let tmp_msg = format!("ERROR: No execution_id found");
        //     error!("{}", tmp_msg);

        //     return Err(tmp_msg);
        // } else {
        //     tmp_execution_id = in_json_message.parameters["execution_id"].as_u64().unwrap() as u32;

        //     // Create a copy and remove the execution id
        //     tmp_request = in_json_message.clone();
        //     // Set as null
        //     tmp_request.parameters["execution_id"].take();
        // }

        // let mut response_json_message = InternalResponseMessage::new();

        // response_json_message.response.msg_id      = in_json_message.response.msg_id.clone();
        // response_json_message.response.msg_code_id = in_json_message.response.msg_code_id;
        // response_json_message.execution_id         = in_json_message.execution_id;
        // response_json_message.wait_flag            = true;
        // response_json_message.response             = in_json_message.response.clone();

        //let received_execution_id : u32 = response_json_message["execution_id"].as_u64().unwrap() as u32;
        //let received_answer : String = String::from( response_json_message["msg_buffer"].as_str().unwrap() );

        {
            TASK_MANAGER.write().unwrap()
                .set_answer_completed(
                    in_json_message.execution_id,
                    in_json_message.response.to_string(),
                )
                .unwrap();
        }

        // Create response
        Ok(in_json_message.clone())
    }

    /**
     * If we do not execute child.status(), the process will be a zombie
     * wait
        try_wait
        wait_with_output

        kill
     */
    pub fn check_status_all_modules(&self) {
        let m = ModuleManager::current();
        let mut tmp_data = m.data.write().unwrap();

        for a_module in tmp_data.list_running_modules.iter_mut() {
            // Send a GetStatus message to all modules
            // let get_status_message = GetStatusMessage {
            //    header :  ApiMessage {
            //       msg_code_id:        String::from("get_status"),
            //       authentication_key: String::from(""),
            //    },
            //    user_id: String::from(""),
            // };

            //  let get_status_message = json!({
            //     "msg_code_id" : "get_status",
            //     "authentication_key" : "xyzzy",
            //     "user_id": "magic_user",
            //  });
            let mut get_status_message: RestRequest = RestRequest::new();

            get_status_message.msg_code_id = String::from("get_status");
            //get_status_message.user_id             = String::from("modules_manager");

            // Send the message
            let json_message = Message::from(&serde_json::to_vec(&get_status_message).unwrap());

            // Send the message to the main control loop
            // Message is a JSON
            debug!("sending message: {}", get_status_message.to_string());

            let status = a_module.push_socket.send(json_message);
            if let Err(e) = status {
                let error_msg = format!(
                    "Error when sending JSON messae to Module: {} Error: {}",
                    a_module.definition.name,
                    nng::Error::from(e)
                );
                error!("{}", error_msg);

                // We are going to assume, the module has crashed. Get exit status of module
                a_module.status = EnumModuleStatus::ERRONEOUS;

                // Get exit status and kill the module
                self.kill_module(a_module);

                // TODO: Remove instance from the module

                for _i in 0..2u32 {
                    self.run_module(a_module);
                }
            } else {
                // Read response message
                let response_message = a_module.push_socket.recv();

                if let Err(e) = response_message {
                    error!(
                        "Error when receiving a message: {} from Module: {}",
                        e, a_module.definition.name
                    );
                    continue;
                }

                // As u8[]
                let json_buffer = response_message.unwrap();

                debug!(
                    "Received message: {}",
                    String::from_utf8(json_buffer.as_slice().to_vec()).unwrap()
                );

                // Decode JSON
                let json_message = serde_json::from_slice(json_buffer.as_slice());
                let json_message: GetStatusResponseStruct = match json_message {
                    Ok(msg) => msg,
                    Err(e) => {
                        let tmp_msg = format!(
                            "ERROR: Unable to decode JSON message: {}. IGNORED",
                            e.to_string()
                        );
                        error!("{}", tmp_msg.as_str());
                        continue;
                    }
                };

                if json_message.status != String::from("Running") {
                    error!(
                        "Module: {} is in an incorrect status: {}",
                        a_module.definition.name, json_message.status
                    );
                }
            }
        }
    }

    /**
     * A module is ready to be used
     */
    pub fn module_is_ready(&self, in_json_message: &InternalResponseMessage) -> Result<InternalResponseMessage, String> {
        info!("Module is ready module JSON: {}", in_json_message.to_string());

        // Check if the message has the module instance id, if not, then generate an error
        if in_json_message.response.result["module_instance_id"].is_null() == true {
            let error_msg = format!("Module Get Status response does not contain the instance id");
            error!("{}", error_msg);
            return Err(error_msg);
        }
        
        let read_instance_id : u32 = in_json_message.response.result["module_instance_id"].as_u64().unwrap() as u32;
               
        if in_json_message.response.result["status"].is_null() == true {
            let error_msg = format!("Module Get Status response does not contain the status field");
            error!("{}", error_msg);
            return Err(error_msg);
        }
        
        let module_status = in_json_message.response.result["status"].as_str().unwrap();
        if module_status == "Ready" {
            let mut tmp_data = self.data.write().unwrap();

            // Search for module and create PUSH socket
            for (_i, current_module) in tmp_data.list_running_modules.iter_mut().enumerate() {
                if current_module.instance_id == read_instance_id {
                    // Try to connect to module 3 times
                    let mut error_flag = false;
                    let mut error_msg = String::new();

                    let push_socket_address = current_module.push_address.clone() + &String::from(":") + 
                                            &current_module.push_port.to_string();

                    for i in 0..3 as u8 {
                        // Give the module time to start
                        thread::sleep(Duration::from_secs(1));

                        info!("   Iteration: {}", i);
                        error_flag = false;

                        // Create the PUSH socket
                        let tmp_push_socket = Socket::new(Protocol::Push0);
                        current_module.push_socket = match tmp_push_socket {
                            Ok(s) => {
                                info!("PUSH Socket to {} correctly created ", current_module.definition.name);
                                s
                            }
                            Err(e) => {
                                error_msg = format!("Unable to create PUSH Socket to {}. Error: {}",
                                    current_module.definition.name,
                                    e.to_string()
                                );
                                error!("{}", error_msg.as_str());

                                error_flag = true;

                                continue;
                            }
                        };

                        // Start listening. It does need the module to be able to connect to. So, it must be up and running
                        let unused_result = current_module.push_socket.dial(push_socket_address.as_str());
                        if let Err(e) = unused_result {
                            let error_msg = format!(
                                "Error when connecting to PUSH socket using address: {}\n Error: {}",
                                push_socket_address, e
                            );

                            error!("{}", error_msg);
                            error_flag = true;
                            continue;
                        } else {
                            // End of the loop
                            break;
                        }
                    }

                    if error_flag == true {
                        // Set the status
                        current_module.status = EnumModuleStatus::ERRONEOUS;

                        // Kill the module
                        if current_module.child_process.is_some() {
                            error!("Module: {} will be killed", current_module.definition.name);

                            // Get exit status and kill the module
                            self.kill_module(current_module);
                        }

                        return Err(error_msg);
                    } else {
                        info!("Correctly connected to PUSH socket. Address: {}", push_socket_address);

                        // Set the status
                        current_module.status = EnumModuleStatus::RUNNING;
                    }

                    break;
                }
            }
        } else {
            info!("Module with instance id: {} is not ready yet. Skipped", read_instance_id);
        }

        Ok(in_json_message.clone())
    }

    /**
     * Try to stop all modules. If not possible, kill them
     */
    pub fn kill_all_modules(&self) {
        info!("Stopping all modules");

        let tmp_data = self.data.read().unwrap();

        for a_module in tmp_data.list_running_modules.iter() {
            info!("Stopping module: {}", a_module.definition.name);

            // Send Exit message to the module
            //  let exit_message = json!({
            //     "msg_code_id" : "exit",
            //     "authentication_key" : "00998844",
            //     "exit_code": "XYZZY"
            //  });

            let mut exit_message: RestRequest = RestRequest::new();

            exit_message.msg_code_id = String::from("exit");
            exit_message.authentication_key = String::from("00998844");
//            exit_message.user_id = String::from("modules_manager");
            exit_message.msg_id = format!("{}", Utc::now().timestamp());
            exit_message.parameters = json!( { "exit_code": "XYZZY" } );

            // Send the message
            let json_message = Message::from(&serde_json::to_vec(&exit_message).unwrap());

            // Send the message to the main control loop
            // Message is a JSON
            debug!("sending message: {}", exit_message.to_string());

            /*

            let status =  a_module.req_socket.send(json_message);
            if let Err(e) = status {
               let error_msg = format!("Error when sending JSON messae to Module: {} Error: {}", a_module.definition.name, nng::Error::from(e) );
               error!("{}", error_msg);

               // We are going to assume, the module has crashed. Get exit status of module
               a_module.status = EnumModuleStatus::ERRONEOUS;

               // Get exit status and kill the module
               kill_module(&mut a_module);

            } else {
               // Read response message
               let response_message = a_module.req_socket.recv();

               if let Err(e) = response_message {
                     error!("Error when receiving a message: {} from Module: {}", e, a_module.definition.name);
                     continue;
               }

               // As u8[]
               let json_buffer = response_message.unwrap();

               debug!("Received message: {}", String::from_utf8( json_buffer.as_slice().to_vec()).unwrap() );

               // Decode JSON
               let json_message = serde_json::from_slice(json_buffer.as_slice());
               let json_message : GetStatusMessageResponse = match json_message {
                     Ok(msg) => msg,
                     Err(e) => {
                        let tmp_msg = format!("ERROR: Unable to decode JSON message: {}. IGNORED", e.to_string());
                        error!("{}", tmp_msg.as_str() );
                        continue;
                     },
               };

               if json_message.status != String::from("Running") {
                  error!("Module: {} is in an incorrect status: {}", a_module.definition.name, json_message.status);
               }

            }
            */
        }
    }

    /**
     * Kill a module
     * First, it tries to read the exit status. If not posible, it will kill it
     */
    fn kill_module(&self, in_module: &mut Module) {
        info!("Killing child process: {} of Module: {}", in_module.instance_id, in_module.definition.name);

        // Not null
        if let Some(c) = &mut in_module.child_process {
            // Try to recover exit status of the process
            match c.try_wait() {
                Ok(Some(status)) => {
                    info!("Module: {} Child: {} exited with status: {}",
                        in_module.definition.name,
                        c.id(),
                        status);
                }
                Ok(None) => {
                    info!("Module: {}  Child: {} has failed. It will be killed",
                        in_module.definition.name,
                        c.id());

                    c.kill().unwrap();
                }
                Err(e) => {
                    error!(
                        "Unable to obtain module status. Module: {}  Child: {} . It will be killed",
                        in_module.definition.name,
                        c.id()
                    );
                    error!("{}", e);
                    c.kill().unwrap();
                }
            }

            // Remove the pid file
            let pid_file_name = in_module.definition.name.clone() + &String::from(".pid");
            debug!("Module PID file: {}", pid_file_name);

            fs::remove_file(pid_file_name.as_str()).expect("Unable to remove module PID file");

            in_module.status = EnumModuleStatus::STOPPED;
            in_module.child_process = None;
        }
    }
}
