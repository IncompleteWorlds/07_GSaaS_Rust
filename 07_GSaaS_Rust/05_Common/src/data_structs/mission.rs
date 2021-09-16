/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * This file contains the definition of Common Structures used in all components
 */

// JSON serialization
use serde::{Deserialize, Serialize};

use chrono::{Utc};


#[derive(Debug, Deserialize, Serialize)]
pub struct Mission
{
    pub id:              String,
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub launch_date:     Option<String>, 
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String, 
} 

impl Mission 
{
    /**
     * Create an empty new mission
     */
    pub fn new() -> Self 
    {
        Mission {
            id:             String::new(),
            name:           String::new(),
            description:    String::new(),
            launch_date:    None,
            // created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
        }
    }
}


