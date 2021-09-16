/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * This file contains the definition of Common Structures used in all components
 */


pub struct Satellite
{
    pub id:              String,
    pub mission_id:      String,
    pub name:            String,
    pub description:     String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub launch_date:     Option<String>, 
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String, 
}