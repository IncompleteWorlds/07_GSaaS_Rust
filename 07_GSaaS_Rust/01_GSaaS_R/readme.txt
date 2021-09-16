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


TODO
========================


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


Date and Time format and parse
=============================================
let dt = Utc.ymd(2018, 1, 26).and_hms_micro(18, 30, 9, 453_829);
assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Millis, false),
           "2018-01-26T18:30:09.453+00:00");
assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Millis, true),
           "2018-01-26T18:30:09.453Z");
assert_eq!(dt.to_rfc3339_opts(SecondsFormat::Secs, true),
           "2018-01-26T18:30:09Z");


======================================================
.wrap_fn(|req, srv| {
    let unauth: Box<dyn IntoFuture<Item = ServiceResponse>> = Box::new(ServiceResponse::new(req.into_parts().0, HttpResponse::Unauthorized().finish()));
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        None => unauth,
        Some(value) => {
            let token = value.to_str().unwrap();
            let mut users = state.users.lock().unwrap();
            let user_state = users.iter_mut().find(|x| x.auth.token == token);
            match user_state {
                None => unauth,
                Some(user) => {
                    Box::new(srv.call(req).map(|res| res))
                }
            }
        }
    }
})





Generate a random key
============================================
// Generate random secret key
        let mut tmp_secret_key = [0u8; 16];
        let mut rng = rand::thread_rng();

        for i in 0..16 {
            let x : u8 = rng.gen_range(0..255);

            tmp_secret_key[i] = x;
        }
        let new_secret_key = encode_hex(&tmp_secret_key);