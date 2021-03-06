/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Modules Manager
 * It executes the messages
 */

use std::result::Result;
use std::fs::File;
use std::process::{Command, Child, Stdio};
use std::{thread, time, time::Duration, thread::JoinHandle};
use std::sync::{Arc, Mutex, Condvar};

// Log 
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Date & Time
use chrono::{DateTime, Utc};

// New Nanomsg
use nng::*;

// Messages
use crate::fds_messages::*;

// Common functions
use crate::common::*;


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
   definition:        VariableDefinition,
   database:          String,
   db_table_name:     String,
   db_column_name:    String,
}

#[derive(Serialize, Deserialize)]
struct VariableDefinition {
   name:             String,
   description:      String,
   default_value:    String,
   // integer, unsigned, bool, float, double, string
   value_type:       EnumVariableType,
}


#[derive(Serialize, Deserialize)]
struct ModuleDefinition {
   name:                String,
   description:         String,
   module_type:         EnumModuleType,
   binary_file:         String,
   binary_file_path:    String,
   working_directory:   String,
   config_file:         String,
   arguments:           String,
   messages:            Vec<String>,
   input_variables:     Vec<VariableDefinition>,
   output_variables:    Vec<OutputVariableDefinition>,
}

/**
 * It describres a Running Module
 * Each module can have several instances running in Parallel
 */
struct Module {
   definition:          ModuleDefinition,

   id:                  u32,

   // TODO: Add a list of instances
   //       Add functions for adding or removing instances from that list

   instance_id:         u32,
   push_address:        String,
   push_port:           u32,
   push_socket:         nng::Socket,
   sub_address:         String,
   req_address:         String,
   status:              EnumModuleStatus,
   // UTC time. Format;   yyyy-mm-ddThh:mi:ss.sss
   start_time:          DateTime<Utc>,
   stop_time:           DateTime<Utc>,
   // Process, after fork
   child_process:       Option<Child>,
}

//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
struct ExecutionRecord {
   execution_id:          u32,
   user_id:               u32,
   module_id:             u32,
   module_instance_id:    u32,
   start_time:            String,
   stop_time:             String,
   status:                EnumExecutionStatus,
   answer:                String,
   pair:                  Arc<(Mutex<bool>, Condvar)>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExecutionRecordModuleAnswer {
   execution_id:          u32,
   user_id:               u32,
   module_id:             u32,
   module_instance_id:    u32,
   answer:                String,
}



/**
 * Definition of the Module Manager pseudo-class
 */
pub struct  ModuleManager {
   // Private
   // ---------------------
   list_running_modules:      Vec<Module>,
   list_executions:           Vec<ExecutionRecord>,
   executions_counter:        u32,

   // Public
   // ---------------------
}

impl ModuleManager
{
   // This function will create an instance of the Module Manager 
   pub fn init() -> ModuleManager 
   {
      ModuleManager {
         list_running_modules:     Vec::new(),
         list_executions:          Vec::new(),
         executions_counter:       0,
      }
   }


   /**
    *  Load the list of modules from the JSON config file
      */
   pub fn load_module_definitions(&mut self, in_config_variables: &ConfigVariables) -> std::result::Result<(), String> 
   {      
      let module_file_name = in_config_variables.modules_definition_file.clone();

      // Open the file and return it. If an error, return the error
      let module_file = File::open( &module_file_name ).unwrap();
      debug!("Modules definition file read: {}", module_file_name.as_str());
      
      let the_definitions  = serde_json::from_reader(module_file);
      let tmp_list : Vec<ModuleDefinition> = match the_definitions {
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

      for a_definition in tmp_list
      {
         let tmp_pull_port = in_config_variables.modules_base_pull_port.trim().parse::<u32>().expect("Incorrect PULL base port number");

         let mut new_module = Module {
            definition:          a_definition,

            id:                  next_id,
            instance_id:         1,

            push_address:        in_config_variables.modules_base_pull_address.clone(),
            push_port:           (tmp_pull_port + next_id),

            push_socket:         nng::Socket::new( nng::Protocol::Push0 ).unwrap(),
            
            sub_address:         in_config_variables.fds_nng_sub_address.clone(),

            req_address:         in_config_variables.fds_nng_rep_address.clone(),

            status:              EnumModuleStatus::IDLE,
            
            // UTC time. Format;   yyyy-mm-ddThh:mi:ss.sss
            start_time:          Utc::now(),
            stop_time:           Utc::now(),
            
            // Process
            child_process:       None,
         };

         info!("   Loading module: {} - {}", new_module.definition.name, new_module.id);

         // Do some checkings
         if new_module.definition.binary_file.is_empty() == true {
            let error_msg = format!("Binary file name is empty");

            error!("{}", error_msg);

            return Err(error_msg);
         }

         // Run the module
         let _tmp_result = run_module(&mut new_module);
         if let Err(e) = _tmp_result  {
            let error_msg = format!("Unable to execute module: {} error: {}", new_module.definition.binary_file.as_str(), e);
            
            error!("{}", error_msg);

            //If there is an error, the module will be added to the list anyway but in an erroneous state
         }
         
         // Add module to the list
         self.list_running_modules.push(new_module);

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
   pub fn call_module(&mut self, in_json_message: &Value) -> Result<String, String> 
   {
      info!("Processing message JSON: {}", in_json_message.to_string() );

      let mut module_found = false;
      let mut received_response : String = String::from("");
      let tmp_msg_code_id = String::from( in_json_message["msg_code_id"].as_str().unwrap() );
      
/*
      // Look for the module that can execute the message
      for a_module in self.list_running_modules.iter_mut() {
         if a_module.definition.messages.contains(&tmp_msg_code_id) == true {

            // Check if module is running
            if a_module.status != EnumModuleStatus::RUNNING {
               info!("Module is not running. It will be started");
   
               if let Err(e) = run_module(a_module) {
                  return Err( build_api_answer_str_json(true, e.to_string().as_str(), "") );
               }
            }
   
            let current_time = Utc::now();
   
            // Next counter
            self.executions_counter += 1;
            // Create a copy
            let current_counter = self.executions_counter;
   
            // Create an execution record
            let exec_record = ExecutionRecord {
               execution_id:          self.executions_counter,
               // TODO
               // in_user_id
               user_id:               111,
               module_id:             a_module.id,
               module_instance_id:    a_module.instance_id,
               start_time:            format!("{}", current_time.naive_utc()),
               stop_time:             format!("{}", current_time.naive_utc()),
               status:                EnumExecutionStatus::RUNNING,
               answer:                String::from(""),
               pair:                  Arc::new((Mutex::new(false), Condvar::new())),
            };
   
            // Save execution record
            debug!("Creating execution record {:?}", exec_record);
            self.list_executions.push(exec_record);
   
            // Get Execution record index
            let mut execution_record_index = 0;
   
            for (i, current_execution) in self.list_executions.iter().enumerate() {
               if current_execution.execution_id == current_counter {
                  execution_record_index = i;
                  break;
               }
            }
   
            // Send messages
            let tmp_message = Message::from( &serde_json::to_vec(in_json_message).unwrap() ); 
   
            // Send the message to the main control loop
            // Message is a JSON
            debug!("sending message: {}", in_json_message.to_string() );
   
            let status =  a_module.push_socket.send(tmp_message);
            if let Err(e) = status {
               let error_msg = format!("Error when sending JSON message to Module: {} Error: {}", 
                                       a_module.definition.name, nng::Error::from(e) );
               error!("{}", error_msg);
   
               return Err( build_api_answer_str_json(true, error_msg.as_str(), "") );
            }
   
            // Wait for the result to be received
            let pair = Arc::new((Mutex::new(false), Condvar::new()));
            let (lock, cvar) = &*pair;
            let mut received_flag = lock.lock().unwrap();

            self.list_executions[execution_record_index].pair = pair.clone();
            
            // as long as the value inside the `Mutex<bool>` is `false`, we wait
            loop {
               // TODO: Define the timeout
               // It will block the current thread
               info!("Blocked waiting for answer ...");
               let result = cvar.wait_timeout(received_flag, Duration::from_secs(60)).unwrap();

               // 60 seconds have passed, or maybe the value changed!
               received_flag = result.0;
               if *received_flag == true {
                  // We received the notification and the value has been updated, we can leave.
                  received_response = self.list_executions[execution_record_index].answer.clone();
                  info!("Received response: {}", received_response);
                  break
               }
            }
   
            self.list_executions[execution_record_index].stop_time = format!("{}", Utc::now().naive_utc() );
   
            // TODO: Save the Execution record in a Database
   
            // Remove execution from the list
            self.list_executions.remove(execution_record_index);
   
            // Return the received response
            module_found = true;
            break;
         }
      }
      
      if module_found == false {
         let error_msg = format!("Error: Module to process message: {} not found", tmp_msg_code_id );
         error!("{}", error_msg);
         
         return Err( build_api_answer_str_json(true, error_msg.as_str(), "") );
      }

      Ok( build_api_answer_str_json(false, "", received_response.as_str()) )
      */



      // Look for the module that can execute the message
      let mut module_index = 0;

      for (i, a_module) in self.list_running_modules.iter().enumerate() {

         if a_module.definition.messages.contains(&tmp_msg_code_id) == true {
            module_index = i;

            // Return the received response
            module_found = true;
            break;
         }
      }

      if module_found == false {
         let error_msg = format!("Error: Module to process message: {} not found", tmp_msg_code_id );
         error!("{}", error_msg);

         return Err( build_api_answer_str_json(true, error_msg.as_str(), "") );
      }

      // Get the module
      let mut tmp_a_module = self.list_running_modules.get_mut(module_index);
      let a_module = match tmp_a_module {
         Some(m) => m,
         None => {
            let error_msg = format!("Internal Error: Module found, but None found. Index: {}", module_index);
            error!("{}", error_msg);
   
            return Err( build_api_answer_str_json(true, error_msg.as_str(), "") );
         },
      };

      // Check if module is running
      if a_module.status != EnumModuleStatus::RUNNING {
         info!("Module is not running. It will be started");

         if let Err(e) = run_module(a_module) {
            return Err( build_api_answer_str_json(true, e.to_string().as_str(), "") );
         }
      }

      let current_time = Utc::now();
   
      // Next counter
      self.executions_counter += 1;
      // Create a copy
      let current_counter = self.executions_counter;

      // Create an execution record
      let exec_record = ExecutionRecord {
         execution_id:          self.executions_counter,
         // TODO
         // in_user_id
         user_id:               111,
         module_id:             a_module.id,
         module_instance_id:    a_module.instance_id,
         start_time:            format!("{}", current_time.naive_utc()),
         stop_time:             format!("{}", current_time.naive_utc()),
         status:                EnumExecutionStatus::RUNNING,
         answer:                String::from(""),
         pair:                  Arc::new((Mutex::new(false), Condvar::new())),
      };

      // Save execution record
      debug!("Creating execution record {:?}", exec_record);
      self.list_executions.push(exec_record);

      // Get Execution record index
      let mut execution_record_index = 0;

      for (i, current_execution) in self.list_executions.iter().enumerate() {
         if current_execution.execution_id == current_counter {
            execution_record_index = i;
            break;
         }
      }

      // Send messages
      let tmp_message = Message::from( &serde_json::to_vec(in_json_message).unwrap() ); 

      // Send the message to the main control loop
      // Message is a JSON
      debug!("sending message: {}", in_json_message.to_string() );

      let status =  a_module.push_socket.send(tmp_message);
      if let Err(e) = status {
         let error_msg = format!("Error when sending JSON message to Module: {} Error: {}", 
                                 a_module.definition.name, nng::Error::from(e) );
         error!("{}", error_msg);

         return Err( build_api_answer_str_json(true, error_msg.as_str(), "") );
      }


      let _resp = thread::spawn(move || {
         // Wait for the result to be received
         let pair = Arc::new((Mutex::new(false), Condvar::new()));
         let (lock, cvar) = &*pair;
         let mut received_flag = lock.lock().unwrap();

         self.list_executions[execution_record_index].pair = pair.clone();
         
         // as long as the value inside the `Mutex<bool>` is `false`, we wait
         loop {
            // TODO: Define the timeout
            // It will block the current thread
            info!("Blocked waiting for answer ...");
            let result = cvar.wait_timeout(received_flag, Duration::from_secs(60)).unwrap();

            // 60 seconds have passed, or maybe the value changed!
            received_flag = result.0;
            if *received_flag == true {
               // We received the notification and the value has been updated, we can leave.
               received_response = self.list_executions[execution_record_index].answer.clone();
               info!("Received response: {}", received_response);
               break
            }
         }

         self.list_executions[execution_record_index].stop_time = format!("{}", Utc::now().naive_utc() );

         // TODO: Save the Execution record in a Database

         // Remove execution from the list
         self.list_executions.remove(execution_record_index);

         Ok( build_api_answer_str_json(false, "", received_response.as_str()) )
      });

      // It should not reach this code
      Ok( String::from("") )
   }

   /**
    * Call a module
      * The module will receive a message and execute it. It will return a message either with the output 
      * or a link to the file containing the answer
      *
      * Return: A JSON object either describing the answer or a link to the resource containing the answer
      */
   pub fn handle_module_answer(&mut self, in_json_message: &Value) -> Result<u32, String> 
   {
      info!("Processing answer from module JSON: {}", in_json_message.to_string() );

      // Deserialize input message
      let response_json_message  = serde_json::from_value( in_json_message.clone() );
      let response_json_message : ExecutionRecordModuleAnswer = match response_json_message {
            Ok(msg) => msg,
            Err(e) => {
               let tmp_msg = format!("ERROR: Incorrect SUB answer message. Error: {}", e.to_string());
               error!("{}", tmp_msg.as_str() );
               
               return Err(tmp_msg);
            },
      };

      // Get Execution record index
      for current_execution in self.list_executions.iter_mut() {
         // Update status and store the answer
         if current_execution.execution_id == response_json_message.execution_id &&
            current_execution.user_id == response_json_message.user_id &&
            current_execution.module_id == response_json_message.module_id &&
            current_execution.module_instance_id == response_json_message.module_instance_id {

            current_execution.status = EnumExecutionStatus::RESPONSE_RECEIVED;
            current_execution.answer = response_json_message.answer; 

            // This block is to ensure the lock is released as soon as possible
            {
               // Notify the conditional variable, the answer has been received
               let (lock, cvar) = &*current_execution.pair;

               let mut received_flag = lock.lock().unwrap();
               *received_flag = true;
               cvar.notify_all();
            }

            info!("Response received for Execution Id: {} Module Id: {}  Module Instance Id: {}", 
                  response_json_message.execution_id,
                  response_json_message.module_id,
                  response_json_message.module_instance_id );

            break;
         }
      }

      // If we reach this point, it means the execution has not be found
      error!("Execution not found. Execution Id: {} Module Id: {}  Module Instance Id: {}", 
               response_json_message.execution_id, response_json_message.module_id, response_json_message.module_instance_id);

      Ok(0)
   }


   /**
    * If we do not execute child.status(), the process will be a zombie
    * wait
       try_wait
       wait_with_output

       kill
    */
   fn check_status_all_modules(&mut self) 
   {
      for a_module in self.list_running_modules.iter_mut() {
         // Send a GetStatus message to all modules
         // let get_status_message = GetStatusMessage {
         //    header :  ApiMessage {
         //       msg_code_id:        String::from("get_status"),
         //       authentication_key: String::from(""),
         //    },  
         //    user_id: String::from(""),
         // };

         let get_status_message = json!({
            "header": {
               "msg_code_id" : "get_status",
               "authentication_key" : "xyzzy"
            },
            "user_id": "magic_user"
         });

         // Send the message
         let json_message = Message::from( &serde_json::to_vec(&get_status_message).unwrap() ); 

         // Send the message to the main control loop
         // Message is a JSON
         debug!("sending message: {}", get_status_message.to_string() );

         let status =  a_module.push_socket.send(json_message);
         if let Err(e) = status {
            let error_msg = format!("Error when sending JSON messae to Module: {} Error: {}", a_module.definition.name, nng::Error::from(e) );
            error!("{}", error_msg);

            // We are going to assume, the module has crashed. Get exit status of module
            a_module.status = EnumModuleStatus::ERRONEOUS;

            // Get exit status and kill the module
            kill_module(a_module);

            // TODO: Remove instance from the module

            for i in 0..2 {
               run_module(a_module);

            }


         } else {
            // Read response message
            let response_message = a_module.push_socket.recv();
            
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
      }
   }

   /**
    * Try to stop all modules. If not possible, kill them
    */
   pub fn kill_all_modules(&self)
   {
      info!("Stopping all modules");

      for a_module in self.list_running_modules.iter() {
         info!("Stopping module: {}", a_module.definition.name);

         // Send Exit message to the module
         let exit_message = json!({
            "msg_code_id" : "exit",
            "authentication_key" : "00998844",
            "exit_code": "XYZZY"
         });

         // Send the message
         let json_message = Message::from( &serde_json::to_vec(&exit_message).unwrap() ); 

         // Send the message to the main control loop
         // Message is a JSON
         debug!("sending message: {}", exit_message.to_string() );

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
}

/**
 * Kill a module
 * First, it tries to read the exit status. If not posible, it will kill it
 */
fn kill_module(in_module: &mut Module) 
{
   info!("Killing child process: {} of Module: {}", in_module.instance_id, in_module.definition.name);

   // Not null
   if let Some(c) = &mut in_module.child_process {
      // Try to recover exit status of the process  
      match c.try_wait() {
         Ok(Some(status)) => {
            info!("Module: {} Child: {} exited with status: {}", in_module.definition.name, c.id(), status);
         },
         Ok(None) => {
            info!("Module: {}  Child: {} has failed. It will be killed", in_module.definition.name, c.id());
            c.kill().unwrap();
         },
         Err(e) => {
            error!("Unable to obtain module status. Module: {}  Child: {} . It will be killed", 
                  in_module.definition.name, c.id());
            error!("{}", e);
            c.kill().unwrap();
         },
      }

      in_module.status = EnumModuleStatus::STOPPED;
      in_module.child_process = None;
   }

}

/**
 * Spawn a process for executing an external module
* Stores the PID of the module
* Set the start time of the process 

TODO: Rename to add_instance
It shall start an instance

*/
fn run_module(in_module: &mut Module) -> Result<u32, String> 
{
   debug!("Running module: {}", in_module.definition.name);

   let mut command_line : String = String::from("");
   
   if in_module.definition.binary_file_path.is_empty() == false {
      command_line.push_str( in_module.definition.binary_file_path.as_str() );
      command_line.push_str("/");
   } 
   command_line.push_str(in_module.definition.binary_file.as_str());

   let tmp_pull_address : String = in_module.push_address.clone() + &String::from(":") + 
                                    &in_module.push_port.to_string();

   let mut tmp_config_file : String = String::from("config.json");

   if in_module.definition.config_file.is_empty() == false {
      tmp_config_file = in_module.definition.config_file.clone();
   } 

   // Command line:
   //   executable  config_file.json   instance_id   pull_address   sub_address   req_address
   let mut child_process = Command::new( command_line.as_str() );

   if in_module.definition.working_directory.is_empty() == false {
      child_process.current_dir( in_module.definition.working_directory.clone() );
   }

   if in_module.definition.arguments.is_empty() == false {
      child_process.arg( in_module.definition.arguments.clone() );
   }
   
   let child_process = child_process.arg( tmp_config_file )                        
                  .arg( in_module.instance_id.to_string()  )
                  .arg( tmp_pull_address )
                  .arg( in_module.sub_address.clone() )
                  .arg( in_module.req_address.clone() )

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
         error!("{}", e );

         // Set the status
         in_module.status = EnumModuleStatus::ERRONEOUS;

         return Err(error_msg);
      }
   };

   // Move the Child
   in_module.child_process = Some(tmp_result);

   // Set the start time as the current time
   //new_module.start_time = format!("{}", now.naive_utc());
   //new_module.stop_time  = format!("{}", now.naive_utc());
   in_module.start_time = Utc::now();
   in_module.stop_time  = Utc::now();

   // Small sleep 1 sec
   thread::sleep(Duration::from_secs(1));

   // Create the PUSH socket 
   let tmp_push_socket = Socket::new(Protocol::Push0);
   in_module.push_socket = match tmp_push_socket {
      Ok(s) =>  { 
         info!("PUSH Socket to {} correctly created ", in_module.definition.name);
         s
      },
      Err(e) => {
         // Set the status
         in_module.status = EnumModuleStatus::ERRONEOUS;

         let error_msg = format!("Unable to create PUSH Socket to {}. Error: {}", in_module.definition.name, e.to_string()); 
         error!("{}", error_msg.as_str());

         // Kill the module
         if in_module.child_process.is_some() {
            error!("Module: {} will be killed", in_module.definition.name);

            // Get exit status and kill the module
            kill_module(in_module);
         }
         
         return Err(error_msg);
      },
   };

   let push_socket_address = in_module.push_address.clone() + &String::from(":") + &in_module.push_port.to_string();

   // Start listening. It does need the module to be able to connect to. So, it must be up and running   
   let unused_result = in_module.push_socket.dial( push_socket_address.as_str() );
   if let Err(e) = unused_result {
      // Set the status
      in_module.status = EnumModuleStatus::ERRONEOUS;

      let error_msg = format!("Error when connecting to PUSH socket using address: {}\n Error: {}", 
                              push_socket_address, e );

      error!("{}", error_msg);

      // Kill the module
      if in_module.child_process.is_some() {
         error!("Module: {} will be killed", in_module.definition.name);

         // Get exit status and kill the module
         kill_module(in_module);
      }
      
      return Err(error_msg);
   } else {
      info!("Correctly connected to PUSH socket. Address: {}", push_socket_address);

      // Set the status
      in_module.status = EnumModuleStatus::RUNNING;
   }

   Ok(0)
}

