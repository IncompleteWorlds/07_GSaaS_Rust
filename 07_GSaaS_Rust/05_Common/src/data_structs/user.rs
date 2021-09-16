/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main - Common
 * 
 * This file contains the definition of Common Structures used in all components
 */

use std::str;

// JSON serialization
use serde::{Deserialize, Serialize};

// Utc, time
use chrono::{DateTime, Duration, Utc};

// UUID
use uuid::Uuid;

use crate::data_structs::license::*;

// Common functions
// use crate::common::*;
// use crate::common_messages::*;


#[derive(Clone)]
pub enum EnumUserRoles {
    ReadOnly, 
	Normal, 
	Administrator,

	MissionOperator, 
	MissionAdministrator,
}

impl EnumUserRoles {
    pub fn to_string(&self) -> String {
        match *self {
            EnumUserRoles::ReadOnly      => String::from("ReadOnly"),
            EnumUserRoles::Normal         => String::from("Normal"),
            EnumUserRoles::Administrator  => String::from("Administrator"),

            EnumUserRoles::MissionOperator       => String::from("MissionOperator"),
            EnumUserRoles::MissionAdministrator  => String::from("MissionAdministrator"),
        }
    }

    // pub fn from_string(in_license: &str) -> Self 
    // {
    //     match in_license {
    //         "Demo"           => EnumLicenseType::DemoLicense,
    //         "Education"      => EnumLicenseType::EducationLicense,
    //         "Community"      => EnumLicenseType::CommunityLicense,
    //         "Professional"   => EnumLicenseType::ProfessionalLicense,
    //         _                => EnumLicenseType::DemoLicense,
    //     }
    // }
}


// Note: u32 cannot be used. It has to be i32

#[derive(Debug, Deserialize, Serialize)]
pub struct User 
{
    pub id:              String,
    pub username:        String,
    // Hashed password. So, it is not stored in clear
    pub password:        String,
    pub email:           String,
    // License type
    // Demo, Education, Community, Professional
    pub license_id:      String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:         String,

    pub role_id:         String,
} 

impl User 
{
    /**
     * Create an empty new user
     */
    pub fn new() -> Self 
    {
        let new_uuid = Uuid::new_v4().to_hyphenated().to_string();

        User {
            id:             new_uuid,
            username:       String::new(),
            password:       String::new(),
            email:          String::new(),
            license_id:     EnumLicenseType::DemoLicense.to_string(),
            // created:        chrono::Local::now().naive_local(),
            created:        Utc::now().to_rfc3339(),
            role_id:        EnumUserRoles::Normal.to_string(),
        }
    }
}


