/**
 * (c) Incomplete Worlds 2021 
 * Alberto Fernandez (ajfg)
 * 
 * FDS as a Service main
 * 
 * This file contains the definition of Common Structures used in all components 
 */
 

// JSON serialization
use serde::{Deserialize, Serialize};
//use serde_json::{json, Value};




#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EnumLicenseType 
{
    DemoLicense,
    EducationLicense,
    CommunityLicense,
    ProfessionalLicense,
}

impl EnumLicenseType 
{
    pub fn to_string(&self) -> String 
    {
        match *self 
        {
            EnumLicenseType::DemoLicense             => String::from("Demo"),
            EnumLicenseType::EducationLicense        => String::from("Education"),
            EnumLicenseType::CommunityLicense        => String::from("Community"),
            EnumLicenseType::ProfessionalLicense     => String::from("Professional"),
        }
    }

    pub fn from_string(in_license: &str) -> Self 
    {
        match in_license {
            "Demo"           => EnumLicenseType::DemoLicense,
            "Education"      => EnumLicenseType::EducationLicense,
            "Community"      => EnumLicenseType::CommunityLicense,
            "Professional"   => EnumLicenseType::ProfessionalLicense,
            _                => EnumLicenseType::DemoLicense,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct License {
    pub id:                String,
    // Demo, Education, Community, Professional
    pub license:           String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub created:           String,
    // Format:  YYYY-MM-DDTHH:MM:SS
    pub expire_at:         String,  
}
