/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * It will implement the entry point and the REST API
 */

use crate::common::*;

// Date & Time
use chrono::{DateTime, Utc};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};



#[derive(Serialize, Deserialize, Clone)]
pub struct BaseFrame {
    pub id:                u32,
    pub creation_time:     String,
}

#[derive(Clone)]
pub struct ReceivedFrame {
    pub header:            BaseFrame,
    pub data:              Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessedFrame {
    pub header:            BaseFrame,
    // List of data from the different layers
    pub data:              Vec<ProcessedLayerFrame>,
}

impl ProcessedFrame {
    pub fn new() -> Self
    {
        ProcessedFrame {
            header:     BaseFrame {
                id:             0,
                creation_time:  Utc::now().to_rfc3339(),
            },
            data:       Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessedLayerFrame {
    pub name:              String,
    pub layer_data:        Vec<FrameFieldValue>,
}

impl ProcessedLayerFrame {
    pub fn new() -> Self
    {
        ProcessedLayerFrame {
            name:           String::from(""),
            layer_data:     Vec::new(),
        }
    }

    pub fn add_field(&mut self, in_name: &str, in_value: &String, in_ool_flag: bool, in_crc_flag: bool, 
                     in_pdu_flag: bool)
    {
        let new_field = FrameFieldValue::new_field(in_name, in_value, in_ool_flag, in_crc_flag, in_pdu_flag);

        self.layer_data.push(new_field);
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct FrameFieldValue {
    pub name:               String,
    pub value:              String,
    pub ool_flag:           bool,
    pub crc_flag:           bool,
    pub pdu_flag:           bool,
    // Frame Field Definition Type Id
    // At the moment, not used
    field_type_id:          u32,
    // Position within the frame
    index:                  u16,
}

impl FrameFieldValue {
    pub fn new() -> Self
    {
        FrameFieldValue {
            name:               String::from(""),
            value:              String::from(""),
            ool_flag:           false,
            crc_flag:           false,
            pdu_flag:           false,
            field_type_id:      0,
            index:              0,
        }
    }

    pub fn new_field(in_name: &str, in_value: &String, in_ool_flag: bool, in_crc_flag: bool, 
               in_pdu_flag: bool) -> Self
    {
        FrameFieldValue {
            name:               String::from(in_name),
            value:              in_value.clone(),
            ool_flag:           in_ool_flag,
            crc_flag:           in_crc_flag,
            pdu_flag:           in_pdu_flag,
            field_type_id:      0,
            index:              0,
        }
    }

}





#[derive(Serialize, Deserialize, Clone)]
// Aka Parameter
pub struct FrameFieldType {
    pub id:                     u32,

    pub name:                   String,

    pub raw_type:               u8,
    pub engineering_type:       u8,
    pub engineering_units:      String,
    pub raw_flag:               bool,
    pub crc_flag:               bool,
    // pub crc_algorithm:        enum,
    pub calibration_id:         u8,
    // Length of parameter in bits. Padding to right
    pub length:                 u32,
    // Index of the field inside of the sequence that composes the frame
    pub sequence_number:        u16,
    pub pdu_flag:               bool,

    // pub ranges:              Vec<Range>,
    // pub limits:              Vec<Limit>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FrameConditionType {
    pub id:                 u32,

    pub parameter_id:       String,

    // Value the parameter must have
    pub parameter_value:    String,
}

#[derive(Serialize, Deserialize, Clone)]
// Aka Container
pub struct FrameType {
    pub id:                     u32,

    pub name:                   String,
    // Frame that contains this frame
    pub parent_id:              u32,
    pub description:            String,
    pub hktm_flag:              bool,

    // Length of frame in bits. Padding to right
    pub frame_length:           u32,

    // Frame is delimited by this marker or only one
    // Ids of the fields
    pub marker_start_id:        u32,
    pub marker_end_id:          u32,

    // Layer identifier to whic this frame is related to
    // 0 = No layer
    pub layer_id:               u32,

    // Expected interval in milliseconds
    pub expected_interval:      u16,

    // Position within parent frame, in bits
    pub relative_position:      u32,

    // Types: 0 - Archived_Frame, 1 - Real_time frame
    // or
    // 0 - Fixed length
    // 1 - Fixed length + marker start
    // 2 - Variable size; marker start, marker end
    pub frame_type:             u8,

    // Default: true
    pub little_endian:          bool,

    // List of conditions identifiers the frame shall meet
    pub conditions:             Vec<u32>,

    // List of parameters identifiers this frame is composed of
    pub fields:                 Vec<u32>,
}
