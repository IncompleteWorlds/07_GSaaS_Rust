/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * Tasks Manager
 * It maintains a list of asynchronous tasks
 * 
 */

use std::sync::{Arc, Mutex, RwLock};
use std::result::Result;
//use std::rc::{Rc};

// Log 
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Date & Time
use chrono::{DateTime, Utc};

// New Nanomsg
//use nng::*;

// Wait for a task to be completed
use crate::wait_for_task::*;

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
   CANCELED,
}


//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
pub struct ExecutionRecord {
   execution_id:          u32,
   user_id:               u32,
   module_id:             u32,
   module_instance_id:    u32,
   start_time:            String,
   stop_time:             String,
   status:                EnumExecutionStatus,
   answer:                String,
   wait_task:             Option<WaitForAnswerFuture>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExecutionRecordModuleAnswer {
   execution_id:          u32,
   user_id:               u32,
   module_id:             u32,
   module_instance_id:    u32,
   answer:                String,
}

struct InternalTaskData
{
    executions_counter:      u32,
    list_executions:         Vec<ExecutionRecord>,
}

impl InternalTaskData
{
    // This function will create an instance of the Module Manager 
    pub fn new() -> Self 
    {
        InternalTaskData {
            executions_counter:   0,
            list_executions:      Vec::new(),
        }
    }
}


pub struct TaskListManager 
{
    data:   RwLock<InternalTaskData>,
}

//lazy_static! {
thread_local! {
    pub static TASK_MANAGER : Arc<TaskListManager> = TaskListManager::new();
    //static TASK_MANAGER : RwLock< Arc<InternalTaskData> >  = RwLock::new( Arc::new( InternalTaskData::new() ) );    
}


impl TaskListManager
{
    // This function will create an instance of the Module Manager 
    pub fn new() -> Arc<TaskListManager>
    {
        Arc::new( TaskListManager{ data: RwLock::new( InternalTaskData::new() ) } )
    }

    // pub fn current_read() -> Arc<TaskListManager> 
    // {
    //     TASK_MANAGER.with(|c| c.read().unwrap().clone() )
    // }
    
    pub fn current() -> Arc<TaskListManager>
    {
        TASK_MANAGER.with(|c| c.clone() )
    }

    // pub fn current_write() -> Arc<TaskListManager> 
    // {
    //     TASK_MANAGER.with(|c| c.write().unwrap().clone() )
    // }
    
    /**
     * Create a new execution record, associate the async task
     * Add to the list of tasks
     * Return the identifier
     */
    pub fn add_task(&self) -> u32
    {
        debug!("Adding task to the list");

        // Create WaitFor task
        let new_wait_task = WaitForAnswerFuture::new();
        
        let current_counter : u32;
        
        {
            let mut tmp_data = self.data.write().unwrap();

            // Next counter
            tmp_data.executions_counter += 1;
            
            // Create a copy
            current_counter = tmp_data.executions_counter;
        }

        // Create execution record
        let current_time = Utc::now();
        
        // Create an execution record
        let exec_record = ExecutionRecord {
            execution_id:          current_counter,
            // TODO
            // in_user_id
            user_id:               111,
            module_id:             0,
            module_instance_id:    0,
            start_time:            format!("{}", current_time.naive_utc()),
            stop_time:             format!("{}", current_time.naive_utc()),
            status:                EnumExecutionStatus::RUNNING,
            answer:                String::from(""),
            wait_task:             Some(new_wait_task), 
        };

        {
            let mut tmp_data = self.data.write().unwrap();
            
            // Save execution record
            debug!("Creating execution record {:?}", exec_record);
            tmp_data.list_executions.push( exec_record );
        }

        // TODO: Save to a database

        debug!("Execution record id: {}", current_counter);

        let tmp_data = self.data.read().unwrap();
        for current_execution in tmp_data.list_executions.iter() {
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

        let tmp_data = self.data.read().unwrap();

        let mut execution_record_index = 0;

        for (i, current_execution) in tmp_data.list_executions.iter().enumerate() {
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
        
        let tmp_data = self.data.read().unwrap();
        
        for current_execution in tmp_data.list_executions.iter() {
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
    pub fn set_answer(&self, in_execution_id: u32, in_answer: String) -> std::result::Result<String, String>
    {
        debug!("Set answer. Execution Id: {}, Answer: {}", in_execution_id, in_answer);

        let mut tmp_data = self.data.write().unwrap();
        
        for current_execution in tmp_data.list_executions.iter_mut() {
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
    pub fn set_completed(&self, in_execution_id: u32) -> std::result::Result<String, String>
    {
        debug!("Set answer completed. Execution Id: {}", in_execution_id);

        let mut tmp_data = self.data.write().unwrap();
        
        for current_execution in tmp_data.list_executions.iter_mut() {
            if current_execution.execution_id == in_execution_id {
                match current_execution.wait_task.as_mut() {
                    Some(t)  => t.set_completed(),
                    None => {
                        let tmp_error_msg = format!("Asynchronous task not found: {}. Ignored", in_execution_id);
                        
                        error!("{}", tmp_error_msg);
                        return Err( tmp_error_msg);
                    },
                };
        
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
    pub fn set_answer_completed(&self, in_execution_id: u32, in_answer: String) -> std::result::Result<String, String>
    {
        debug!("Set answer completed. Execution Id: {}, Answer: {}", in_execution_id, in_answer);

        let mut tmp_data = self.data.write().unwrap();

        debug!("Number records: {} ", tmp_data.list_executions.len());
        
        for current_execution in tmp_data.list_executions.iter_mut() {
            debug!("current_execution.execution_id = {}   in_execution_id = {}", current_execution.execution_id,  in_execution_id);
            if current_execution.execution_id == in_execution_id {
                current_execution.answer = in_answer;

                match current_execution.wait_task.as_mut() {
                    Some(t)  => t.set_completed(),
                    None => {
                        let tmp_error_msg = format!("Asynchronous task not found: {}. Ignored", in_execution_id);
        
                        error!("{}", tmp_error_msg);
                        return Err( tmp_error_msg);
                    },
                };
        
                return Ok( String::from("") );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    /**
     * Wait asynchronously until the task is completed
     */
    pub async fn wait_for_task(&self, in_execution_id: u32) -> std::result::Result<String, String>
    {
        debug!("Waiting for tast to complete. Execution Id: {}", in_execution_id);

        let mut tmp_data = self.data.write().unwrap();
        
        for current_execution in tmp_data.list_executions.iter_mut() {
            if current_execution.execution_id == in_execution_id {
                match &mut current_execution.wait_task {
                    Some(t)  => t.await,
                    None => {
                        let tmp_error_msg = format!("Asynchronous task not found: {}. Ignored", in_execution_id);
        
                        error!("{}", tmp_error_msg);
                        return Err( tmp_error_msg);
                    },
                };
        
                return Ok( String::from("") );
            }
        }

        let tmp_error_msg = format!("Execution id not found: {}", in_execution_id);
                
        error!("{}", tmp_error_msg);
        return Err( tmp_error_msg);
    }

    // /**
    // * Remove an execution record from the list
    // */
    // // async fn remove_execution_record(&mut self, in_execution_index: u32)
    // // {
    // //     let i = in_execution_index as usize;
    // //     if i < self.list_executions.len() {
    // //         self.list_executions.remove(i);
    // //     }
    // // }

    // /**
    // * Set the stop time of an execution
    // */
    // // async fn set_stop_time(&mut self, in_execution_index: u32)
    // // {
    // //     let i = in_execution_index as usize;
    // //     if i < self.list_executions.len() {
    // //         self.list_executions[in_execution_index as usize].stop_time = format!("{}", Utc::now().naive_utc() );
    // //     }
    // // }
}
