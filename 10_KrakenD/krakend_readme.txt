Using KrakenD

API Designer
https://designer.krakend.io/#!/



Check the syntax of your krakend.json is good
Syntax checking 
$krakend check --config krakend.json --debug 


Run KrakenD
Start the server 
$krakend run -c krakend.json -d 

$ krakend run --config /path/to/krakend.json -p 8080 


Help
------------------

$ krakend 
 
`7MMF' `YMM'                  `7MM                         `7MM"""Yb. 
  MM   .M'                      MM                           MM    `Yb. 
  MM .d"     `7Mb,od8 ,6"Yb.    MM  ,MP'.gP"Ya `7MMpMMMb.    MM     `Mb 
  MMMMM.       MM' "'8)   MM    MM ;Y  ,M'   Yb  MM    MM    MM      MM 
  MM  VMA      MM     ,pm9MM    MM;Mm  8M""""""  MM    MM    MM     ,MP 
  MM   `MM.    MM    8M   MM    MM `Mb.YM.    ,  MM    MM    MM    ,dP' 
.JMML.   MMb..JMML.  `Moo9^Yo..JMML. YA.`Mbmmd'.JMML  JMML..JMMmmmdP' 
_______________________________________________________________________ 
 
Version: 1.4.1 
 
The API Gateway builder 
 
Usage: 
  krakend [command] 
 
Available Commands: 
  check       Validates that the configuration file is valid. 
  help        Help about any command 
  run         Run the KrakenD server. 
 
Flags: 
  -c, --config string   Path to the configuration filename 
  -d, --debug           Enable the debug 
  -h, --help            help for krakend 
 
Use "krakend [command] --help" for more information about a command. 
