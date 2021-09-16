/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * 
 */

//use std::vec;
use std::str;

use std::io::{Cursor, Read};
use bitstream_io::{BigEndian, LittleEndian, BitReader};

// Log 
use log::{debug, error, info, trace, warn};

// Serialize/Deserialize; YAML, JSON
use serde::{Serialize, Deserialize};
//use serde_json::{json, Value};

use crate::layers::*;
use crate::common::*;
use crate::frames::*;

/*
The frames are composed of:
- Header:
- Telemetry Transfer Frame
- CLTU - TC

The content of the Header and Tail depends on the message type
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct NisLayer {
    pub name:             String,

    // By default true
    little_endian_flag:   bool,

    // Common to both versions
    packet_size:          u32,
    sc_id:                u16,
    data_stream_type:     u8,
    vc_id:                u8,
    route_id:             u16,
    sequence_flag:        u8,
    quality_flag:         u8,
    
    // Version 0
    ERT:                  u64,
    
    // Version 1
    // Version 0 or 1 = 0x81
    du_version:           u8,
    ERT_format:           u8,
    annotation_length:    u8,
    ERT_v1:               [u8;10],
    // max size = 128
    annotation:           Vec<u8>,

    data:                 Vec<u8>,
}
  
impl NisLayer
{
    // TODO: Set a default value
    pub fn new(in_little_endian_flag : bool) -> Self
    {
        NisLayer {
            name:                  String::from("nis_layer"),
            little_endian_flag:    in_little_endian_flag,

            // Common to both versions
            packet_size:           0,
            sc_id:                 0,
            data_stream_type:      0,
            vc_id:                 0,
            route_id:              0,
            sequence_flag:         0,
            quality_flag:          0,
            
            // Version 0
            ERT:                   0,

            // Version 1
            du_version:            0,
            ERT_format:            0,
            annotation_length:     0,
            ERT_v1:                [0;10],
            // max size = 128
            annotation:            Vec::new(),

            data:                  Vec::new(),
        }
    }

    // ========================================================
    // Internal functions

    fn process_v0(&mut self, in_first_byte: u8, in_reader_le: &mut BitReader<&mut std::io::Cursor<&&[u8]>, bitstream_io::LittleEndian>, 
                  in_reader_be: &mut BitReader<&mut std::io::Cursor<&&[u8]>, bitstream_io::BigEndian>) -> Result<u32, String>
    {
        self.du_version = 0;

        // Packet size
        let mut tmp_4_buffer: [u8; 4] = [0, 0, 0, 0];
        let mut tmp_3_buffer: [u8; 3] = [0, 0, 0];

        if self.little_endian_flag == true {            
            in_reader_le.read_bytes(&mut tmp_3_buffer).unwrap();

            tmp_4_buffer[0] = in_first_byte;
            tmp_4_buffer[1] = tmp_3_buffer[0];
            tmp_4_buffer[2] = tmp_3_buffer[1];
            tmp_4_buffer[3] = tmp_3_buffer[2];

            self.packet_size = u32::from_le_bytes(tmp_4_buffer);
        } else {
            in_reader_be.read_bytes(&mut tmp_3_buffer).unwrap();

            tmp_4_buffer[3] = in_first_byte;
            tmp_4_buffer[2] = tmp_3_buffer[0];
            tmp_4_buffer[1] = tmp_3_buffer[1];
            tmp_4_buffer[0] = tmp_3_buffer[2];

            self.packet_size = u32::from_be_bytes(tmp_4_buffer);
        }
        debug!("NIS Layer. DU size; {}", self.packet_size);

        // S/C ID
        if self.little_endian_flag == true {
            self.sc_id = in_reader_le.read::<u16>(1).unwrap();
        } else {
            self.sc_id = in_reader_be.read::<u16>(1).unwrap();
        }

        // Data Stream type
        if self.little_endian_flag == true {
            self.data_stream_type = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.data_stream_type = in_reader_be.read::<u8>(1).unwrap();
        }
        
        // VC ID
        if self.little_endian_flag == true {
            self.vc_id = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.vc_id = in_reader_be.read::<u8>(1).unwrap();
        }

        // Route ID
        if self.little_endian_flag == true {
            self.route_id = in_reader_le.read::<u16>(1).unwrap();
        } else {
            self.route_id = in_reader_be.read::<u16>(1).unwrap();
        }

        // ERT
        if self.little_endian_flag == true {
            self.ERT = in_reader_le.read::<u64>(1).unwrap();
        } else {
            self.ERT = in_reader_be.read::<u64>(1).unwrap();
        }

        // Sequence flag
        if self.little_endian_flag == true {
            self.sequence_flag = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.sequence_flag = in_reader_be.read::<u8>(1).unwrap();
        }

        // Quality flag
        if self.little_endian_flag == true {
            self.quality_flag = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.quality_flag = in_reader_be.read::<u8>(1).unwrap();
        }

        // Data
        let data_length : u16 = 20 - (self.packet_size as u16 - 1);
        debug!("NIS Layer. User data size: {}   DU size; {}", data_length, self.packet_size);

        let mut b;
        for _i in 0..data_length {
            if self.little_endian_flag == true {
                b = in_reader_le.read::<u8>(1).unwrap();
            } else {
                b = in_reader_be.read::<u8>(1).unwrap();
            }
            self.data.push(b);
        }

        Ok(0)
    }

    fn process_v1(&mut self, in_reader_le: &mut BitReader<&mut std::io::Cursor<&&[u8]>, bitstream_io::LittleEndian>, 
                  in_reader_be: &mut BitReader<&mut std::io::Cursor<&&[u8]>, bitstream_io::BigEndian>) -> Result<u32, String>
    {
        self.du_version = 1;

        // Packet size
        if self.little_endian_flag == true {
            self.packet_size = in_reader_le.read::<u32>(1).unwrap();
        } else {
            self.packet_size = in_reader_be.read::<u32>(1).unwrap();
        }

        // S/C ID
        if self.little_endian_flag == true {
            self.sc_id = in_reader_le.read::<u16>(1).unwrap();
        } else {
            self.sc_id = in_reader_be.read::<u16>(1).unwrap();
        }

        // Data Stream type
        if self.little_endian_flag == true {
            self.data_stream_type = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.data_stream_type = in_reader_be.read::<u8>(1).unwrap();
        }

        // VC ID
        if self.little_endian_flag == true {
            self.vc_id = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.vc_id = in_reader_be.read::<u8>(1).unwrap();
        }

        // Route ID
        if self.little_endian_flag == true {
            self.route_id = in_reader_le.read::<u16>(1).unwrap();
        } else {
            self.route_id = in_reader_be.read::<u16>(1).unwrap();
        }

        // Sequence flag
        if self.little_endian_flag == true {
            self.sequence_flag = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.sequence_flag = in_reader_be.read::<u8>(1).unwrap();
        }

        // Quality flag
        if self.little_endian_flag == true {
            self.quality_flag = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.quality_flag = in_reader_be.read::<u8>(1).unwrap();
        }

        // ERT Format
        if self.little_endian_flag == true {
            self.ERT_format = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.ERT_format = in_reader_be.read::<u8>(1).unwrap();
        }

        // Private annotation length
        if self.little_endian_flag == true {
            self.annotation_length = in_reader_le.read::<u8>(1).unwrap();
        } else {
            self.annotation_length = in_reader_be.read::<u8>(1).unwrap();
        }

        // ERT
        if self.little_endian_flag == true {
            if in_reader_le.read_bytes(&mut self.ERT_v1).is_err() {
                let msg = format!("Nis Layer. Error reading ERT: {:?}", self.ERT_v1);
                error!("{}", msg);
                return Err(msg);
            }
        } else {
            if in_reader_be.read_bytes(&mut self.ERT_v1).is_err() {
                let msg = format!("Nis Layer. Error reading ERT: {:?}", self.ERT_v1);
                error!("{}", msg);
                return Err(msg);
            }
        }

        // Private annotation
        let mut b;
        if self.little_endian_flag == true {
            for _i in 0..self.annotation_length {
                b = in_reader_le.read::<u8>(1).unwrap();
                self.annotation.push(b);
            }
        } else {
            for _i in 0..self.annotation_length {
                b = in_reader_be.read::<u8>(1).unwrap();
                self.annotation.push(b);
            }
        }

        // Data
        let ERT_length : u16 = match self.ERT_format {
            41 => 8,
            42 => 10,
            // Default
            _ => 10,
        };
        let data_length : u16 = (15 + ERT_length + self.annotation_length as u16) - 
                                (self.packet_size as u16 - 1);

        debug!("NIS Layer. User data size: {}   DU size; {}", data_length, self.packet_size);
        for _i in 0..data_length {
            if self.little_endian_flag == true {
                b = in_reader_le.read::<u8>(1).unwrap();
            } else {
                b = in_reader_be.read::<u8>(1).unwrap();
            }
            self.data.push(b);
        }
        Ok(0)
    }

    fn receive_v0(&mut self, out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>
    {
        // Packet size
        out_frame.add_field("packet_size", &self.packet_size.to_string(), false, false, false);

        // S/C ID
        out_frame.add_field("sc_id", &self.sc_id.to_string(), false, false, false);

        // Data Stream type
        out_frame.add_field("data_stream_type", &self.data_stream_type.to_string(), false, false, false);

        // VC ID
        out_frame.add_field("vc_id",  &self.vc_id.to_string(), false, false, false);

        // Route ID
        out_frame.add_field("route_id",  &self.route_id.to_string(), false, false, false);

        // ERT
        out_frame.add_field("ERT",  &self.ERT.to_string(), false, false, false);

        // Sequence flag
        out_frame.add_field("sequence_flag",  &self.sequence_flag.to_string(), false, false, false);

        // Quality flag
        out_frame.add_field("quality_flag",  &self.quality_flag.to_string(), false, false, false);

        // Data
        out_frame.add_field("data", &String::from_utf8(self.data.clone()).unwrap(), false, false, true);

        Ok(0)
    }

    fn receive_v1(&mut self, out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>
    {
        // Packet size
        out_frame.add_field("packet_size", &self.packet_size.to_string(), false, false, false);

        // S/C ID
        out_frame.add_field("sc_id", &self.sc_id.to_string(), false, false, false);

        // Data Stream type
        out_frame.add_field("data_stream_type", &self.data_stream_type.to_string(), false, false, false);

        // VC ID
        out_frame.add_field("vc_id",  &self.vc_id.to_string(), false, false, false);

        // Route ID
        out_frame.add_field("route_id",  &self.route_id.to_string(), false, false, false);

        // Sequence flag
        out_frame.add_field("sequence_flag",  &self.sequence_flag.to_string(), false, false, false);

        // Quality flag
        out_frame.add_field("quality_flag",  &self.quality_flag.to_string(), false, false, false);

        // ERT Format
        out_frame.add_field("ERT_format",  &self.ERT_format.to_string(), false, false, false);

        // Private annotation length
        out_frame.add_field("private_annotation_length",  &self.annotation_length.to_string(), false, false, false);

        // ERT
        let mut tmp_str = String::from( str::from_utf8(&self.ERT_v1).unwrap() );
        out_frame.add_field("ERT", &tmp_str, false, false, false);

        // Private annotation
        tmp_str = String::from_utf8(self.annotation.clone()).unwrap();
        out_frame.add_field("private_annotation",  &tmp_str, false, false, false);

        // Data
        out_frame.add_field("data", &String::from_utf8(self.data.clone()).unwrap(), false, false, true);
        
        Ok(0)
    }
}

impl LayerTrait for NisLayer {
    fn get_name(&self) -> String
    {
        self.name.clone()
    }

    // Set the configuration parameters of this type of Layer
    fn load_configuration(&mut self, in_config_variables: &ConfigVariables)
    {
        info!("Nis load configuration. Not implemented yet");
    }

    fn process(&mut self, in_buffer: &[u8]) -> Result<u32, String>
    {
        let mut cursor_le = Cursor::new(&in_buffer);
        let mut cursor_be = Cursor::new(&in_buffer);

        let mut reader_le = BitReader::endian(&mut cursor_le, LittleEndian);
        let mut reader_be = BitReader::endian(&mut cursor_be, BigEndian);
        
        // DU type
        let mut header_type: u8 = 0;
        if self.little_endian_flag == true {
            header_type = reader_le.read::<u8>(1).unwrap();
        } else {
            header_type = reader_be.read::<u8>(1).unwrap();
        }

        if header_type == 0x81 {
            return self.process_v1(&mut reader_le, &mut reader_be);
        } else {
            return self.process_v0(header_type, &mut reader_le, &mut reader_be);
        }
    }

    fn receive(&mut self, in_buffer: &[u8], out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>
    {
        // Process the frame
        if let Err(e) = self.process(in_buffer) {
            return Err(e);
        }

        out_frame.name = self.name.clone();

        // Copy value to output processed frame
        if self.du_version == 0 {
            return self.receive_v0(out_frame);
        } else {
            return self.receive_v1(out_frame);
        }
    }

    fn get_payload(&self) -> Vec<u8>
    {
        self.data.clone()
    }

}