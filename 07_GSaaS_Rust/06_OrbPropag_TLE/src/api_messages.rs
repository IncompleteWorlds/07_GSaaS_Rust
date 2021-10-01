/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service
 * Data structures used in the TM Decoder
 */
// use std::result::Result;

use chrono::NaiveDateTime;
// Serialize/Deserialize; YAML, JSON
use serde::{Deserialize, Serialize};
// use serde_json;

//use std::collections::HashMap;


// =======================================================
// Orbit Propagation using SGP4 / TLE
// =======================================================

/**
 * Propagate a satellite orbit using the SGP4 / TLE propagator
 */
 #[derive(Serialize, Deserialize, Debug)]
 pub struct OrbPropagationTleStruct {
 
     pub mission_id:            String,
     pub satellite_id:          String,
 
     pub add_to_database:       bool,
 
     pub epoch_format:          String,
 
     // 2020-05-15T11:30:00.000"
     pub start_time:            String,
     pub stop_time:             String,
 
     pub step_size:             u16,
 
     pub initial_position:      [f64; 3],
     pub initial_velocity:      [f64; 3],
 
     pub input:                 InputTleStruct,
     
     pub output:                OutputTleStruct,    
 }
 
 /**
  * List of satellite ephemeris
  */
 #[derive(Serialize, Deserialize, Debug)]
 pub struct OrbPropagationTleResponseStruct {
   
     pub mission_id:          String,
     pub satellite_id:        String,
 
     pub reference_frame:     String,
     pub epoch_format:        String,
     
     pub ephemeris:           Vec<SatelliteStateVector>,
 }



// #[derive(Serialize, Deserialize, Debug)]
// pub struct Vector3Struct {
//     pub x:       f64,
//     pub y:       f64,
//     pub z:       f64,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct InputTleStruct {
    pub tle: TleStruct,
    // TODO
   // pub OMM: OmmStruct,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TleStruct {
    pub name:    Option<String>,
    pub line1:   String,
    pub line2:   String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OmmStruct {
    pub object_name:   String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputTleStruct {

    pub reference_frame:      String,
    pub interpolation_order:  u16,
    pub output_format:        String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SatelliteStateVector {
    pub time:                  String,
    pub position:              [f64; 3],
    pub velocity:              [f64; 3],
}


