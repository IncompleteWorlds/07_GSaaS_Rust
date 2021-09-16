/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 * 
 * GS as a Service main
 * 
 * DB Module definition
 */

pub mod schema;
pub mod http_access;
pub mod users;

//#[macro_use]
use diesel;

use diesel::sqlite::SqliteConnection;

use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

 
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/**
 *
 */
// pub fn establish_connection() -> SqliteConnection 
// {
//     if cfg!(test) {
//         let conn = SqliteConnection::establish(":memory:")
//           .unwrap_or_else(|_| panic!("Error creating test database"));
        
//         //let _result = diesel_migrations::run_pending_migrations(&conn);

//         conn
//     } else {
//         dotenv().ok();
    
//         let database_url = "data/users.db";
    
//         SqliteConnection::establish(&database_url)
//           .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
//     }
// }


// pub fn run_migrations(conn: &SqliteConnection) {
//   let _ = diesel_migrations::run_pending_migrations(&*conn);
// }

/**
 * Establish a connection with the SQLite database and create a pool of connections
 */
// let conn = establish_connection().get().unwrap();
pub fn establish_connection() -> DbPool 
{
    let manager;

    if cfg!(test) {
        manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        //let pool = r2d2::Pool::builder().build(manager).expect("Failed to create DB pool.");
        
        //run_migrations(&pool.get().unwrap());

        //pool
    } else {
        //dotenv().ok();
    
        let database_url = "data/fdsaas.db"; 
        
        manager = ConnectionManager::<SqliteConnection>::new(database_url);
    }

    r2d2::Pool::builder().build(manager).expect("Failed to create DB pool.")
}