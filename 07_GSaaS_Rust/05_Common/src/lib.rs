/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * GS as a Service 
 * Common functions
 */


pub mod common_messages;
pub mod common;
pub mod claims;
pub mod data_structs;
pub mod http_errors;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
