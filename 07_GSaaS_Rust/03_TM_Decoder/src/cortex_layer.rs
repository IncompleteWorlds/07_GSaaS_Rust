/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * CORTEX TM Message Data processor

NOTE: CORTEX Telemetry
frames are 32-bit aligned (LSBs of the last word are zero-filled if the frame or block length, in bytes,
is not a multiple of 4. This is the source for the 1-byte zero fill).


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

/*
The frames are composed of:
- Header:
  - Marker. Fix valueñ 0xabcdefgh

- Tail. Fix value; 0xabcdedfgh

The content of the Header and Tail depends on the message type
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct CortexLayer {
    pub name:             String,

    // By default true
    little_endian_flag:   bool,

    marker_start:         u32,
    marker_end:           u32,

    // Without include the start marker
    msg_size:             u32,
    flow_id:              u32,
    header_data:          Vec<u8>,

    data:                 Vec<u8>,
}
  
impl CortexLayer
{
    // TODO: Set a default value
    pub fn new(in_little_endian_flag : bool) -> Self
    {
        CortexLayer {
            name:                  String::from("cortex_layer"),
            little_endian_flag:    in_little_endian_flag,
            // 1234567890
            marker_start:          0x499602d2,
            msg_size:              0,
            flow_id:               0,
            header_data:           Vec::new(),
            data:                  Vec::new(),
            // -1234567890
            marker_end:            0xb669fd2e, 
        }
    }

}

impl LayerTrait for CortexLayer {
    fn get_name(&self) -> String
    {
        self.name.clone()
    }

    // Set the configuration parameters of this type of Layer
    fn load_configuration(&mut self, in_config_variables: &ConfigVariables)
    {
        info!("Cortex load configuration. Not implemented yet");
    }

    fn process(&mut self, in_buffer: &[u8]) -> Result<u32, String>
    {
        let mut cursor_le = Cursor::new(&in_buffer);
        let mut cursor_be = Cursor::new(&in_buffer);

        let mut reader_le = BitReader::endian(&mut cursor_le, LittleEndian);
        let mut reader_be = BitReader::endian(&mut cursor_be, BigEndian);
        
        
        // Check header. Start mark
        let mut tmp_value: u32;

        if self.little_endian_flag == true {
            tmp_value = reader_le.read::<u32>(1).unwrap()
        } else {
            tmp_value = reader_be.read::<u32>(1).unwrap()
        }
        if tmp_value != self.marker_start {
            let msg = format!("Cortex Layer. Header mark not found. Marker: {}", tmp_value);
            error!("{}", msg);
            return Err(msg);
        }

        // Read the whole header
        if self.little_endian_flag == true {
            tmp_value = reader_le.read::<u32>(1).unwrap()
        } else {
            tmp_value = reader_be.read::<u32>(1).unwrap()
        }
        self.msg_size = tmp_value;

        if self.little_endian_flag == true {
            tmp_value = reader_le.read::<u32>(1).unwrap()
        } else {
            tmp_value = reader_be.read::<u32>(1).unwrap()
        }
        self.flow_id = tmp_value;

        let mut rest_tm_header : [u8; 64] = [0;64];
        if self.little_endian_flag == true {
            reader_le.read_bytes(&mut rest_tm_header).unwrap();
        } else {
            reader_be.read_bytes(&mut rest_tm_header).unwrap();
        }
        // Add bytes to payload data
        for b in rest_tm_header.iter() {
            self.header_data.push(*b);
        }

        // Read payload
        let mut tmp_buffer: [u8; 4] = [0, 0, 0, 0];
        let mut read_marker_end : u32;
        let mut error_in_tail = false;
        let mut counter = 12+64;

        // Frame is 32bits aligned
        loop {
            // Read next 32 bits integer            
            /*
            if self.little_endian_flag == true {
                if reader_le.read_bytes(&mut tmp_buffer).is_err() {
                    error_in_tail = true;
                    break;
                }

                read_marker_end = u32::from_le_bytes(tmp_buffer);
            } else {
                //reader_be.read_bytes(&mut tmp_buffer).unwrap();
                if reader_be.read_bytes(&mut tmp_buffer).is_err() {
                    error_in_tail = true;
                    break;
                }

                read_marker_end = u32::from_be_bytes(tmp_buffer);
            }
            */
            if self.little_endian_flag == true {
                tmp_value = reader_le.read::<u32>(1).unwrap()
            } else {
                tmp_value = reader_be.read::<u32>(1).unwrap()
            }

            counter += 4;
            // Have we read the max length
            if counter >= self.msg_size {
                // Have we found the end marker
                if tmp_value != self.marker_end {
                    error_in_tail = true;
                }
                break;
            }

            // Check if it is end marker (the tail)
            if tmp_value == self.marker_end {
                break;
            } else {
                // Add bytes to payload data
                for b in tmp_buffer.iter() {
                    self.data.push(*b);
                }
            }
        }

        // Check tail
        if error_in_tail == true {
            let msg = format!("Cortex Layer. Tail mark not found. Marker: {}", tmp_value);
            error!("{}", msg);
            return Err(msg);
        }

        Ok(0)
    }

    fn receive(&mut self, in_buffer: &[u8], out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>
    {
        // Process the frame
        if let Err(e) = self.process(in_buffer) {
            return Err(e);
        }

        out_frame.name = self.name.clone();

        // Copy value to output processed frame
        out_frame.add_field("marker_start",  &self.marker_start.to_string(), false, false, false);

        out_frame.add_field("msg_size",      &self.msg_size.to_string(), false, false, false);
        out_frame.add_field("flow_id",       &self.flow_id.to_string(), false, false, false);
        out_frame.add_field("header_data",   &String::from_utf8(self.header_data.clone()).unwrap(), false, false, false);
        out_frame.add_field("data",          &String::from_utf8(self.data.clone()).unwrap(), false, false, true);
       
        out_frame.add_field("marker_end",    &self.marker_end.to_string(), false, false, false);

        Ok(0)
    }

    fn get_payload(&self) -> Vec<u8>
    {
        self.data.clone()
    }

}
