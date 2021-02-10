/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * Generic layer
 * 
 * Variable size layer.
 * Frame is delimited by two markers; start, end
 */

use std::vec;

use std::io::{Cursor, Read};
use bitstream_io::{BigEndian, LittleEndian, BitReader};

// Log 
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use crate::layers::*;
use crate::common::*;
use crate::frames::*;


#[derive(Serialize, Deserialize, Clone)]
pub struct GenericLayer {
    pub name:             String,

    // By default true
    little_endian_flag:   bool,

    marker_start:         u32,
    marker_end:           u32,

    data:                 Vec<u8>,

    // Reference to the frame type
    frame_def:            &FrameType,

    // Parameters that compose this frame
    parameters:           Vec<FrameFieldValue>,
}
  
impl GenericLayer
{
    // TODO: Set a default value
    pub fn new(in_frame_def: &FrameType) -> Self
    {
        GenericLayer {
            name:                  String::from("cortex_layer"),
            little_endian_flag:    in_frame_def.little_endian,

            marker_start:          0,
            data:                  Vec::new(),
            marker_end:            0, 

            frame_def:             in_frame_def,
        }
    }

}

impl LayerTrait for GenericLayer {
    fn get_name(&self) -> String
    {
        self.name.clone()
    }

    fn get_payload(&self) -> Vec<u8>
    {
        self.data.clone()
    }


    // Set the configuration parameters of this type of Layer
    fn load_configuration(&mut self, in_config_variables: &ConfigVariables)
    {
        info!("{} load configuration. Not implemented yet", self.name);
    }

    fn process(&mut self, in_buffer: &[u8]) -> Result<u32, String>
    {
        let mut cursor_le = Cursor::new(&in_buffer);
        let mut cursor_be = Cursor::new(&in_buffer);

        let mut reader_le = BitReader::endian(&mut cursor_le, LittleEndian);
        let mut reader_be = BitReader::endian(&mut cursor_be, BigEndian);

        debug!("Processing frame type: {}", self.frame_def.name);
        
        for field_id in self.frame_def.fields {
            let mut output : FrameFieldValue;

            let field_def = Frame::get_field_definition(field_id);

            
            let mut tmp_value;
            
            if self.little_endian_flag == true {
                tmp_value = reader_le.read::<u32>(1).unwrap()
            } else {
                tmp_value = reader_be.read::<u32>(1).unwrap()
            }

            // Create a new paramater value type
            output.name          = field_def.name.clone();
            output.value         = tmp_value.to_string();
            output.ool_flag      = false;
            output.crc_flag      = field_def.crc_flag;
            output.pdu_flag      = field_def.pdu_flag;
            output.field_type_id = field_def.id;
            output.index         = field_def.sequence_number;

            // Add parameter to the list
            self.parameters.push(output);
        }
        
        // // Check header. Start mark
        // let mut tmp_value: u32;

        // if tmp_value != self.marker_start {
        //     let msg = format!("Cortex Layer. Header mark not found. Marker: {}", tmp_value);
        //     error!("{}", msg);
        //     return Err(msg);
        // }

        // // Read the whole header
        // if self.little_endian_flag == true {
        //     tmp_value = reader_le.read::<u32>(1).unwrap()
        // } else {
        //     tmp_value = reader_be.read::<u32>(1).unwrap()
        // }
        // self.msg_size = tmp_value;

        // if self.little_endian_flag == true {
        //     tmp_value = reader_le.read::<u32>(1).unwrap()
        // } else {
        //     tmp_value = reader_be.read::<u32>(1).unwrap()
        // }
        // self.flow_id = tmp_value;

        // let mut rest_tm_header : [u8; 64] = [0;64];
        // if self.little_endian_flag == true {
        //     reader_le.read_bytes(&mut rest_tm_header).unwrap();
        // } else {
        //     reader_be.read_bytes(&mut rest_tm_header).unwrap();
        // }
        // // Add bytes to payload data
        // for b in rest_tm_header.iter() {
        //     self.header_data.push(*b);
        // }

        // // Read payload
        // let mut tmp_buffer: [u8; 4] = [0, 0, 0, 0];
        // let mut read_marker_end : u32;
        // let mut error_in_tail = false;
        // let mut counter = 12+64;

        // // Frame is 32bits aligned
        // loop {
        //     // Read next 32 bits integer            
        //     /*
        //     if self.little_endian_flag == true {
        //         if reader_le.read_bytes(&mut tmp_buffer).is_err() {
        //             error_in_tail = true;
        //             break;
        //         }

        //         read_marker_end = u32::from_le_bytes(tmp_buffer);
        //     } else {
        //         //reader_be.read_bytes(&mut tmp_buffer).unwrap();
        //         if reader_be.read_bytes(&mut tmp_buffer).is_err() {
        //             error_in_tail = true;
        //             break;
        //         }

        //         read_marker_end = u32::from_be_bytes(tmp_buffer);
        //     }
        //     */
        //     if self.little_endian_flag == true {
        //         tmp_value = reader_le.read::<u32>(1).unwrap()
        //     } else {
        //         tmp_value = reader_be.read::<u32>(1).unwrap()
        //     }

        //     counter += 4;
        //     // Have we read the max length
        //     if counter >= self.msg_size {
        //         // Have we found the end marker
        //         if tmp_value != self.marker_end {
        //             error_in_tail = true;
        //         }
        //         break;
        //     }

        //     // Check if it is end marker (the tail)
        //     if tmp_value == self.marker_end {
        //         break;
        //     } else {
        //         // Add bytes to payload data
        //         for b in tmp_buffer.iter() {
        //             self.data.push(*b);
        //         }
        //     }
        // }

        // // Check tail
        // if error_in_tail == true {
        //     let msg = format!("Cortex Layer. Tail mark not found. Marker: {}", tmp_value);
        //     error!("{}", msg);
        //     return Err(msg);
        // }

        Ok(0)
    }

    fn receive(&mut self, in_buffer: &[u8], out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>
    {
        // // Process the frame
        // if let Err(e) = self.process(in_buffer) {
        //     return Err(e);
        // }

        // out_frame.name = self.name.clone();

        // // Copy value to output processed frame
        // out_frame.add_field("marker_start",  &self.marker_start.to_string(), false, false, false);

        // out_frame.add_field("msg_size",      &self.msg_size.to_string(), false, false, false);
        // out_frame.add_field("flow_id",       &self.flow_id.to_string(), false, false, false);
        // out_frame.add_field("header_data",   &String::from_utf8(self.header_data.clone()).unwrap(), false, false, false);
        // out_frame.add_field("data",          &String::from_utf8(self.data.clone()).unwrap(), false, false, true);
       
        // out_frame.add_field("marker_end",    &self.marker_end.to_string(), false, false, false);

        Ok(0)
    }

}

