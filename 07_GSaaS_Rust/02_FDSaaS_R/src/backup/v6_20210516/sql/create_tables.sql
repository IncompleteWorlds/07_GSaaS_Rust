-- 
-- (c) Incomplete Worlds 2020 
-- Alberto Fernandez (ajfg)
--  
-- FDS as a Service main
--  
-- This file defines all the SQL tables
-- 

-- NOT USED ???
-- may 2021

DROP TABLE IF EXISTS t_user;
CREATE TABLE IF NOT EXISTS t_user (
  id                   CHARACTER(36) NOT NULL PRIMARY KEY,
  username             CHARACTER(40) NOT NULL,
  password             CHARACTER(40) NOT NULL,
  email                VARCHAR(60) NOT NULL,
  license              CHARACTER(40) NOT NULL,
  created              DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TABLE IF EXISTS t_http_access;
CREATE TABLE IF NOT EXISTS t_http_access (
  id                   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  request_time         DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  ip_address           VARCHAR(256)  NOT NULL,
  hostname             VARCHAR(128),
  operation            VARCHAR(256) NOT NULL
);

