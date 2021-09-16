/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service Common
 * 
 * This file contains the definition of Common Structures used in all components
 */

 use core::f64;

 // Time
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono::format::ParseError;

pub struct GroundStation
{
    pub id:         String,
    pub name:       String,
    pub owner:      String,
    pub created:    NaiveDateTime,

    pub antennas:   Vec<Antenna>,
}

pub struct Antenna
{
    pub id:             String,
    pub station_id:     String,
    pub latitude:       f64,
    pub longitude:      f64,
    pub altitude:       f64,
    // Format:  YYYY-MM-DDTHH:MM:SS
    //pub created:        String,
    pub created:        NaiveDateTime,
}