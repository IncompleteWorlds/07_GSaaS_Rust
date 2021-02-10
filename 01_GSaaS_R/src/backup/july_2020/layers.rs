/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * It will implement the entry point and the REST API
 */

use crate::common::*;

use crate::frames::*;


// Trait. Sort of Interface
pub trait LayerTrait {
    fn get_name(&self) -> String;

    fn load_configuration(&mut self, in_config_variables: &ConfigVariables);
    
    fn process(&mut self, in_buffer: &[u8]) -> Result<u32, String>;

    /**
     * Process the received input buffer and add the fields to the processed layer
     * It could require to update its internal state
     */
    fn receive(&mut self, in_buffer: &[u8], out_frame: &mut ProcessedLayerFrame) -> Result<u32, String>;

    fn get_payload(&self) -> Vec<u8>;
}

// pub struct ListLayers {
//     pub layers: Vec<Box<dyn LayerTrait>>,
// }
