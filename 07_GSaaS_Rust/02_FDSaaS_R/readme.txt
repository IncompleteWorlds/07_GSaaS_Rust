Readme
========================
Build
$ cargo build


Check without building
$ cargo check 

Run without arguments
$ cargo run

$ cargo run -- arg1 arg2 arg3

Visualize the doc
$ rustup doc --std

$ rustup doc


USAGE:
    rustup doc <--alloc|--book|--cargo|--core|--edition-guide|--nomicon|--proc_macro|--reference|--rust-by-example|--rustc|
    --rustdoc|--std|--test|--unstable-book|--embedded-book>





HTTP Method	CRUD	Entire Collection (e.g. /users)	Specific Item (e.g. /users/123)
POST	Create	201 (Created), ‘Location’ header with link to /users/{id} containing new ID.	Avoid using POST on single resource
GET	Read	200 (OK), list of users. Use pagination, sorting and filtering to navigate big lists.	200 (OK), single user. 404 (Not Found), if ID not found or invalid.
PUT	Update/Replace	405 (Method not allowed), unless you want to update every resource in the entire collection of resource.	200 (OK) or 204 (No Content). Use 404 (Not Found), if ID not found or invalid.
PATCH	Partial Update/Modify	405 (Method not allowed), unless you want to modify the collection itself.	200 (OK) or 204 (No Content). Use 404 (Not Found), if ID not found or invalid.
DELETE	Delete	405 (Method not allowed), unless you want to delete the whole collection — use with caution.	200 (OK). 404 (Not Found), if ID not found or invalid.


Adding a file to the project
=====================================

Add a line; "mod filename;" to main.response


TODO
========================


CURL Test
=====================================

curl --request POST \
  --url http://localhost:3000/api/invitation \
  --header 'content-type: application/json' \
  --data '{"email":"test@test.com"}'

# dbg! will print something like this in your terminal where you are runnig the app
{
    "id": "67a68837-a059-43e6-a0b8-6e57e6260f0d",
    "email": "test@test.com",
    "expires_at": "2018-10-23T09:49:12.167510"
}

$ curl -i --url http://127.0.0.1:11005/gsaas/

$ curl -i --request POST  --url http://127.0.0.1:11005/gsaas/api/register

These are equivalent
$ curl -i --request PUT  --url http://127.0.0.1:11005/fdsaas/api
$ curl -i --request POST  --url http://127.0.0.1:11005/gsaas/fdsaas/api


Tests:

$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/version"
$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/status"
$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/orb_propagation/usage"

$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/orb_propagation" --header 'content-type: application/json' --data @test-modules/msg_orb_propagation.json 

$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/orb_propagation_tle" --header 'content-type: application/json' --data @test-modules/msg_orb_propagation.json 

$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/run_script" --header 'content-type: application/json' --data @test-modules/run_script_demo.json 

$ curl -i --request PUT --url "http://localhost:11005/fdsaas/api/register" --header 'content-type: application/json' --data @test-modules/register_user_demo.json 


Stop the server:

$ curl -i --request GET --url "http://localhost:11005/fdsaas/api/exit" --data '{"msg_code_id" : "exit", "authentication_key" : "00998844", "exit_code": "XYZZY" }'

$ curl -i --request POST --url "http://localhost:9000/gsaas/exit/XYZZY"



test-modules folder includes several python scripts for testing the API

Example of Cargo
========================

[package]
name = "gsaas"
version = "0.1.0"
authors = ["Alberto Fernandez Garcia (External) <ajfg70@wanadoo.fr>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
log = "0.4.0"
log4rs = "0.10.0"
jwt = "0.4.0"
nng-sys = "0.1.3"
rusqlite = "0.21.0"
chrono="0.4"
yaml = "0.3.0"


Serialize - Deserialize to JSON
====================================

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}


Files Management. Rust 1.26 onwards
========================================
Read a file to a String

use std::fs;

fn main() {
    let data = fs::read_to_string("/etc/hosts").expect("Unable to read file");
    println!("{}", data);
}

Read a file as a Vec<u8>

use std::fs;

fn main() {
    let data = fs::read("/etc/hosts").expect("Unable to read file");
    println!("{}", data.len());
}

Write a file

use std::fs;

fn main() {
    let data = "Some data!";
    fs::write("/tmp/foo", data).expect("Unable to write file");
}

Files Management. Rust 1 onwards
========================================
Read a file to a String

use std::fs::File;
use std::io::Read;

fn main() {
    let mut data = String::new();
    let mut f = File::open("/etc/hosts").expect("Unable to open file");
    f.read_to_string(&mut data).expect("Unable to read string");
    println!("{}", data);
}

Read a file as a Vec<u8>

use std::fs::File;
use std::io::Read;

fn main() {
    let mut data = Vec::new();
    let mut f = File::open("/etc/hosts").expect("Unable to open file");
    f.read_to_end(&mut data).expect("Unable to read data");
    println!("{}", data.len());
}

Write a file

Writing a file is similar, except we use the Write trait and we always write out bytes. You can convert a String / &str to bytes with as_bytes:

use std::fs::File;
use std::io::Write;

fn main() {
    let data = "Some data!";
    let mut f = File::create("/tmp/foo").expect("Unable to create file");
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

Format Strings
=====================================
format!("{} {} {}", fname, n, lname),


Global variables
=====================================
static mut configVariables : BTreeMap<String, String> = BTreeMap::new();




API Gateway
==========================

Requirements

    Incoming rest request with headers

    Analyse the header and choose correct forwarding host

    Forward the dance request from 1 to host chosen in 2

    Handle response from host and send same response to client


Parse parameters of a query in Actix
==========================================

It is also possible to get or query the request for path parameters by name:

async fn index(req: HttpRequest) -> Result<String> {
    let name: String =
        req.match_info().get("friend").unwrap().parse().unwrap();
    let userid: i32 = req.match_info().query("userid").parse().unwrap();

    Ok(format!("Welcome {}, userid {}!", name, userid))
}



Hash
==========================================

let hash: String = Sha256::hash((number * BASE).to_string().as_bytes()).hex();


Session counter
==========================================
fn welcome(req: &HttpRequest) -> Result<HttpResponse> {
    println!("{:?}", req);

    // session
    let mut counter = 1;
    if let Some(count) = req.session().get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        counter = count + 1;
    }

    // set counter to session
    req.session().set("counter", counter)?;

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/welcome.html")))
}

GMAT Console
=====================================
It has a Nanomsg interface that accepts JSON message
It can be installed using $ build/install.sh


GMAT Run a GMAT script
=====================================

{
    "msg_code_id"        : "run_script",
    "authentication_key" : "00998844",
    "user_id"            : "",

    "output_file_name"   : "Output file name, where script results SHALL be written. Otherwise, output will be empty",
    "script_text"        : "Script it self 
    
    this is a line;   \n"
}

Converting to String
=====================================
To convert any type to a String is as simple as implementing the ToString trait for the type. 
Rather than doing so directly, you should implement the fmt::Display trait which automagically
provides ToString and also allows printing the type as discussed in the section on print!.

use std::fmt;

struct Circle {
    radius: i32
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Circle of radius {}", self.radius)
    }
}

fn main() {
    let circle = Circle { radius: 6 };
    println!("{}", circle.to_string());
}


Parsing a String
=====================================

One of the more common types to convert a string into is a number. The idiomatic approach to this 
is to use the parse function and either to arrange for type inference or to specify the type to 
parse using the 'turbofish' syntax. Both alternatives are shown in the following example.

This will convert the string into the type specified so long as the FromStr trait is implemented 
for that type. This is implemented for numerous types within the standard library. To obtain this
functionality on a user defined type simply implement the FromStr trait for that type.

fn main() {
    let parsed: i32 = "5".parse().unwrap();
    let turbo_parsed = "10".parse::<i32>().unwrap();

    let sum = parsed + turbo_parsed;
    println!("Sum: {:?}", sum);
}


Log4 imports
=====================================

// Log
//use log::{debug, error, info, trace, warn};
//use log::{LevelFilter, SetLoggerError};
//use log4rs::append::console::{ConsoleAppender, Target};
//use log4rs::append::file::FileAppender;
//use log4rs::config::{Appender, Config, Root};
//use log4rs::encode::pattern::PatternEncoder;
//use log4rs::filter::threshold::ThresholdFilter;


actix-web. Delays. Long tasks
=====================================

async fn my_handler() -> impl Responder {
    tokio::time::delay_for(Duration::from_secs(5)).await; // <-- Ok. Worker thread will handle other requests here
    "response"
}



actix-web. Response with the content of a file
=====================================

async fn index() -> impl Responder {
    let content = include_str!("index.html");

    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(content)
}




Initialize SQLite Database
=====================================


Following is the basic syntax of sqlite3 command to create a database: −

$ /home/alberto/Projects/07_GSaaS_Rust/02_FDSaaS_R/data/
$ sqlite3 fdsaas.db

sqlite> .database
sqlite> .quit

Create the tables by running:

$ sqlite3  fdsaas.db  < "../src/sql/create_tables.sql" 





The .dump Command

You can use .dump dot command to export complete database in a text file using 
the following SQLite command at the command prompt.

$ sqlite3 testDB.db .dump > testDB.sql

The above command will convert the entire contents of testDB.db database into 
SQLite statements and dump it into ASCII text file testDB.sql. You can perform restoration 
from the generated testDB.sql in a simple way as follows −

$ sqlite3 testDB.db < testDB.sql



sqlite connection string examples
mode = ro, rw, rwc (read, write, create), memory

"file:data.db?mode=ro&cache=private"


Diesel SETUP Database
=====================================

FDSaaS
$ export DATABASE_URL=/home/alberto/Projects/07_GSaaS_Rust/02_FDSaaS_R/data/fdsaas.db
or
$ echo DATABASE_URL=/home/alberto/Projects/07_GSaaS_Rust/02_FDSaaS_R/data/fdsaas.db  > .env
$ echo DATABASE_URL=fdsaas.db  > .env

$ diesel setup --database-url fdsaas.db 

View schemas

$ diesel print-schema --database-url fdsaas.db




$ diesel migration generate create_posts

Creating migrations/2020-07-31-103738_create_posts/up.sql
Creating migrations/2020-07-31-103738_create_posts/down.sql

Edit and fill in those two Files
Run:

$ diesel migration run

$ cd examples/diesel

Install Diesel
$ cargo install diesel_cli --no-default-features --features sqlite

$ echo "DATABASE_URL=test.db" > .env

It will create the database if it does not exist
$ diesel setup

Run the migration, update database if needed, create the src/data/schema.rs
$ diesel migration run



Actix-web Server bind
=====================================


bind  0.0.0.0:nnn  it binds to any Internet address




Rust type conversions String, &str, Vec<u8> and &[u8]
=====================================

```rust
let s:String = ...
let st:&str = ...
let u:&[u8] = ...
let v:Vec<u8> = ...
 
&str    -> String    String::from(st)
&str    -> &[u8]     st.as_bytes()
&str    -> Vec<u8>   st.as_bytes().to_owned()       via &[u8]
String  -> &str      &s                             alt. s.as_str()
String  -> &[u8]     s.as_bytes()
String  -> Vec<u8>   s.into_bytes()
&[u8]   -> &str      str::from_utf8(u).unwrap()
&[u8]   -> String    String::from_utf8(u).unwrap()
&[u8]   -> Vec<u8>   u.to_owned()
Vec<u8> -> &str      str::from_utf8(&v).unwrap()    via &[u8]
Vec<u8> -> String    String::from_utf8(v)
Vec<u8> -> &[u8]     &v
