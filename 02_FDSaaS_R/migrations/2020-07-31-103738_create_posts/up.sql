-- 
-- (c) Incomplete Worlds 2020 
-- Alberto Fernandez (ajfg)
--  
-- FDS as a Service main
--  
-- This file defines all the SQL tables
-- 

CREATE TABLE IF NOT EXISTS t_user (
  id                   CHARACTER(36) NOT NULL PRIMARY KEY,
  username             CHARACTER(40) NOT NULL,
  password             CHARACTER(40) NOT NULL,
  email                VARCHAR(60),
  created_at           DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS t_http_access (
	id	                 INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	request_time	       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	ip_address	         VARCHAR ( 256 ) NOT NULL,
	hostname	           VARCHAR ( 128 ),
	operation            VARCHAR ( 256 ) NOT NULL
);