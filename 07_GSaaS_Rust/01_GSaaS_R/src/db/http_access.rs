/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * 
 * Functions to manage HTTP access; CRUD
 */
// use uuid::Uuid;
use serde::{Deserialize, Serialize};

//#[macro_use]
use diesel;
use diesel::prelude::*;

use crate::schema::*;


#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name="t_http_access"]
pub struct HttpAccess {
     //pub id:            i32,
     pub request_time:  chrono::NaiveDateTime,
     pub ip_address:    String,
     pub hostname:      String,
     pub operation:     String,
}


impl HttpAccess 
{
    pub fn create(conn: &SqliteConnection, in_datetime: chrono::NaiveDateTime, in_address: &String, 
                  in_hostname: &String, in_operation: &String) -> Result<usize, diesel::result::Error>
    {
        // Create a new record
        let new_access = HttpAccessDb {
            // Autoincrement
            //id:              0,
            request_time:    in_datetime.clone(),
            ip_address:      in_address.clone(),
            hostname:        in_hostname.clone(),
            operation:       in_operation.clone(),
        };
        // Insert into the table
        let res = diesel::insert_into(t_http_access::table).values(&new_access)
                                                           .execute(conn);

        match res {
            Ok(n) => return Ok(n),
            Err(e) => return Err(e),
        };
    }
}

