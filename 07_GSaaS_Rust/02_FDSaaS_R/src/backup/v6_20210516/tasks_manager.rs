/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Tasks Manager
 * It maintains a list of asynchronous tasks
 * 
 */

use std::{ops::Add, sync::{Arc, Mutex, RwLock}};
use std::result::Result;
//use std::rc::{Rc};

// Log 
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Date & Time
// To be replaced by std::time
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

// Diesel
#[macro_use]
use diesel;
use diesel::prelude::*;

// Database
use crate::schema::*;

use main::{GLOBAL_DATA};


#[macro_use]
use lazy_static::lazy_static;


// This is need for checking only some values
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumExecutionStatus {
   IDLE,
   RUNNING,
   RESPONSE_RECEIVED,
   COMPLETED,
   STOPPED,
   CANCELLED,
}

impl EnumExecutionStatus {
    fn to_string(&self) -> String {
        match *self {
            EnumExecutionStatus::IDLE               => String::from("Idle"),
            EnumExecutionStatus::RUNNING            => String::from("Running"),
            EnumExecutionStatus::RESPONSE_RECEIVED  => String::from("Response received"),
            EnumExecutionStatus::COMPLETED          => String::from("Completed"),
            EnumExecutionStatus::STOPPED            => String::from("Stopped"),
            EnumExecutionStatus::CANCELLED          => String::from("Cancelled"),
        }
    }

    pub fn from_string(in_status: &str) -> Self {
        match in_status {
            "Idle"               => EnumExecutionStatus::IDLE,
            "Running"            => EnumExecutionStatus::RUNNING,
            "Response received"  => EnumExecutionStatus::RESPONSE_RECEIVED,
            "Completed"          => EnumExecutionStatus::COMPLETED,
            "Stopped"            => EnumExecutionStatus::STOPPED,
            "Cancelled"          => EnumExecutionStatus::CANCELLED,
            _                    => EnumExecutionStatus::IDLE,
        }
    }
}


//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
pub struct ExecutionRecord {
   execution_id:          u32,
   user_id:               String,
   module_id:             u32,
   module_instance_id:    u32,
   start_time:            String,
   stop_time:             String,
   status:                EnumExecutionStatus,
   answer:                String,
   complete_flag:         bool,
}

//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug, Queryable, Insertable)]
#[table_name="t_execution_record"]
pub struct ExecutionRecordDB {
   execution_id:          i32,
   user_id:               String,
   module_id:             i32,
   module_instance_id:    i32,
   start_time:            String,
   stop_time:             String,
   status:                String,
   complete_flag:         bool,
   expiration_time:       String,
}

impl ExecutionRecordDB
{
    pub fn new(in_record : &ExecutionRecord) -> Self
    {
        let output = ExecutionRecordDB {
            execution_id :        in_record.execution_id as i32,
            user_id :             in_record.user_id,
            module_id :           in_record.module_id as i32,
            module_instance_id :  in_record.module_instance_id as i32,
            start_time :          in_record.start_time,
            stop_time :           in_record.stop_time,
            status :              in_record.status.to_string(),
            complete_flag :       in_record.complete_flag,
            expiration_time :     in_record.start_time,
        };

        let mut tmp_start: NaiveDateTime = NaiveDateTime::parse_from_str(in_record.start_time.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();

        // Add +1 hour
        tmp_start = tmp_start.add( FixedOffset::west(3600) );
        
        output.expiration_time     = tmp_start.to_string();

        output
    }

    /**
    * Insert a new User record in the table
    */
    pub fn create(&self, conn: &SqliteConnection)  -> Result<usize, diesel::result::Error>
    {
        // Insert into the table
        match diesel::insert_into(t_execution_record::table).values(self).execute(conn) {
            Ok(n) => {
                Ok(n)
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    /*
    pub fn list(conn: &SqliteConnection) -> Vec<Self> 
    {
        //user_dsl.load::<User>(conn).expect("Error loading users")
    }
    */

    pub fn by_id(conn: &SqliteConnection, in_id: i32) -> Option<Self> 
    {
        if in_id < 0 {
            return None;
        }

        match t_execution_record::table.find(in_id).first::<ExecutionRecordDB>(conn) {
            Ok(r) => Some(r),
            Err(_e) => None,
        }
    }
}




#[derive(Serialize, Deserialize, Debug)]
struct ExecutionRecordModuleAnswer {
   execution_id:          u32,
   user_id:               u32,
   module_id:             u32,
   module_instance_id:    u32,
   answer:                String,
}

pub struct TaskListManager 
{
    executions_counter:      u32,
    list_executions:         Vec<ExecutionRecord>,
}

lazy_static! {
    pub static ref TASK_MANAGER : Arc<RwLock<TaskListManager>> = Arc::new( RwLock::new( TaskListManager::new() ) );  
}

impl TaskListManager
{   
    // This function will create an instance of the Tasks Manager 
    pub fn new() -> Self 
    {
        TaskListManager {
            executions_counter:   0,
            list_executions:      Vec::new(),
        }
    }

    /**
     * Create a new execution record, associate the async task
     * Add to the list of tasks
     * Return the identifier
     */
    pub fn add_task(&mut self, in_module_id : u32, in_instance_id: u32, in_user_id: String) -> u32
    {
        debug!("Adding task to the list");

        let current_counter : u32;
        
        self.executions_counter += 1;

        // Create a copy
        current_counter = self.executions_counter;

        // Create execution record
        let current_time = Utc::now();
        
        // Create an execution record
        let exec_record = ExecutionRecord {
            execution_id:          current_counter,
            user_id:               in_user_id,
            module_id:             in_module_id,
            module_instance_id:    in_instance_id,
            start_time:            format!("{}", current_time.naive_utc()),
            stop_time:             format!("{}", current_time.naive_utc()),
            status:                EnumExecutionStatus::RUNNING,
            answer:                String::from(""),
            //wait_task:             Some(new_wait_task), 
            complete_flag:         false,
        };

        let exec_record_db = ExecutionRecordDB::new(&exec_record);
        
        // Save execution record
        debug!("Creating execution record {:?}", exec_record);
        self.list_executions.push( exec_record );
        
        // Save to a database

        // Obtain a connection to the database
        let new_conn;
            
        {   
            let tmp_data = GLOBAL_DATA.read().unwrap();
            new_conn = tmp_data.db_pool.get().unwrap();
        }
        exec_record_db.create(new_conn);

        debug!("Execution record id: {}", current_counter);

        for current_execution in self.list_executions.iter() {
            debug!("--- Execution record id: {}", current_execution.execution_id);
        }

        return current_counter;
    }

    /**
    * Return an execution record based on its counter
    */
    pub fn get_execution_record(&self, in_execution_id: u32) -> u32
    {
        debug!("Get execution record. Index: {}", in_execution_id);

        let mut execution_record_index = 0;

        for (i, current_execution) in self.list_executions.iter().enumerate() {
            if current_execution.execution_id == in_execution_id {
                execution_record_index = i;
                break;
            }
        }

        debug!("Execution record id: {}", execution_record_index);

        return execution_record_index as u32;
    }

    /**
     * Return the received response from an external module
     */
    pub fn get_answer(&self, in_execution_id: u32) -> std::result::Result<String, String>
    {
        debug!("Get answer. Index: {}", in_execution_id);
        
        for current_execution in self.list_executions.iter() {
            if current_execution.execution_id == in_execution_id {
                return Ok( current_execution.answer.clone() );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Return the received response from an external module
     */
    pub fn set_answer(&mut self, in_execution_id: u32, in_answer: String) -> std::result::Result<String, String>
    {
        debug!("Set answer. Execution Id: {}, Answer: {}", in_execution_id, in_answer);
        
        for current_execution in self.list_executions.iter_mut() {
            if current_execution.execution_id == in_execution_id {
                current_execution.answer = in_answer;
                return Ok( String::from("") );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Set a task as complete. It will be resumed
     */
    pub fn set_completed(&mut self, in_execution_id: u32) -> std::result::Result<String, String>
    {
        debug!("Set answer completed. Execution Id: {}", in_execution_id);
        
        for current_execution in self.list_executions.iter_mut() {
            if current_execution.execution_id == in_execution_id {
                current_execution.complete_flag = true;
        
                return Ok( String::from("") );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Set a task as complete. It will be resumed
     * Also, set the answer for that task
     */
    pub fn set_answer_completed(&mut self, in_execution_id: u32, in_answer: String) -> std::result::Result<String, String>
    {
        debug!("Set answer completed. Execution Id: {}, Answer: {}", in_execution_id, in_answer);

        debug!("Number records: {} ", self.list_executions.len());
        
        for current_execution in self.list_executions.iter_mut() {
            debug!("current_execution.execution_id = {}   in_execution_id = {}", current_execution.execution_id,  in_execution_id);
            if current_execution.execution_id == in_execution_id {
                current_execution.answer = in_answer;
                current_execution.complete_flag = true;

                return Ok( String::from("") );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Check if a task is copmlete
     */
    pub fn is_complete(&self, in_execution_id: u32) -> std::result::Result<bool, String>
    {
        debug!("Is complete. Execution Id: {}", in_execution_id);
        
        for current_execution in self.list_executions.iter() {
            if current_execution.execution_id == in_execution_id {        
                return Ok( current_execution.complete_flag );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Wait asynchronously until the task is completed
     */
    // pub fn wait_for_task(&mut self, in_execution_id: u32) -> std::result::Result<String, String>
    // {
    //     debug!("Waiting for tast to complete. Execution Id: {}", in_execution_id);
                
    //     return Ok( String::from("") );
    // }

    /**
     * This is a dummy function for preventing a compile error
     */
    pub async fn test() 
    {
    }
}
