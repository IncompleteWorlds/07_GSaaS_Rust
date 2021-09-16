-- 
-- (c) Incomplete Worlds 2021
-- Alberto Fernandez (ajfg)
--  
-- FDS as a Service main
--  
-- This file defines all the SQL tables of the TOOLS module. Central DB
-- 

CREATE TABLE IF NOT EXISTS t_user (
  id                   TEXT(36) NOT NULL PRIMARY KEY,
  username             TEXT(40) NOT NULL,
  password             TEXT(40) NOT NULL,
  email                TEXT(60) NOT NULL,
  license_id           TEXT(40) NOT NULL,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  created              TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS t_mission (
  id                   TEXT(10) NOT NULL PRIMARY KEY,
  name                 TEXT(40) NOT NULL,
  description          TEXT(128) NOT NULL,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  created              TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS t_satellite (
  id                   TEXT(10) NOT NULL PRIMARY KEY,
  mission_id           TEXT(10) NOT NULL,
  name                 TEXT(40) NOT NULL,
  description          TEXT(128) NOT NULL,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  launch_date          TEXT,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  created              TEXT NOT NULL,
  -- 1 = Logged in, 0 = Logged out
  logged               INTEGER NOT NULL,

  FOREIGN KEY(mission_id) REFERENCES t_mission(id)
);




CREATE TABLE IF NOT EXISTS t_license (
  id                   TEXT(36) NOT NULL PRIMARY KEY,
  license              TEXT(40) NOT NULL,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  created              TEXT NOT NULL,
  -- Format:  YYYY-MM-DDTHH:MM:SS
  expire_at            TEXT NOT NULL
);

