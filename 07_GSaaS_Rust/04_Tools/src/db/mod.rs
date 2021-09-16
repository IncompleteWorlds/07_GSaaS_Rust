/**
 * (c) Incomplete Worlds 2021
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service - Tools
 *
 * DB Module definition
 */

// List of 'modules' = files that compose the 'db' crate = package/lib
pub mod ground_station;
pub mod antenna;
pub mod mission;
pub mod satellite;
pub mod schema;
pub mod user;


//#[macro_use]
use diesel;

use diesel::sqlite::SqliteConnection;

use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;



/**
 * Establish a connection with the SQLite database and create a pool of connections
 */
// let conn = establish_connection().get().unwrap();
pub fn establish_connection() -> DbPool {
    let manager;

    if cfg!(test) {
        manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        //let pool = r2d2::Pool::builder().build(manager).expect("Failed to create DB pool.");

        //run_migrations(&pool.get().unwrap());

        //pool
    } else {
        //dotenv().ok();

        let database_url = "data/tools.db";

        manager = ConnectionManager::<SqliteConnection>::new(database_url);
    }

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool.")
}
