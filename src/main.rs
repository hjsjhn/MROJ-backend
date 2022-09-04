use actix_web::guard::Connect;
use log;
use std::env;
use env_logger;
// use sqlx::{PgPool, query};
use rusqlite::{params, Connection};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::File;
use dotenv::dotenv;
use std::path::Path;
use std::error::Error;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::result::Result as StdResult;
use actix_web::{middleware::Logger, App, HttpServer, Responder};
use actix_web::web::{self, route, Data};
use std::collections::HashMap;

mod handler;
mod config;
mod error_log;
mod runner;

pub type Result<T = (), E = Box<dyn Error>> = StdResult<T, E>;

#[actix_web::main]
async fn main() -> Result {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let manager = SqliteConnectionManager::file("data.db");
    let pool = Pool::new(manager).unwrap();

    // Create a database table if not exists.
    let conn = pool.get().unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS jobs (id INT, created_time VARCHAR, updated_time VARCHAR, submission_id INT, state VARCHAR, result VARCHAR, score FLOAT, cases INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS submission (id INT, source_code VARCHAR, language VARCHAR, user_id INT, contest_id INT, problem_id INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS cases (jobid INT, caseid INT, result VARCHAR, time INT, memory INT, info VARCHAR)", [])?;
/*
    dotenv().ok();
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
*/

    let config: config::Config = config::parse_from_file("config.json".to_string()).expect("Config file format error.");
    let (address, port) = (config.server.bind_address.to_string(), config.server.bind_port);

    let mut prob_map = HashMap::new();
    for prob in &config.problems {
        prob_map.insert(prob.id, prob.clone());
    }

    let mut stmt = conn.prepare("SELECT * FROM jobs ORDER BY id DESC LIMIT 1;")?; 
    let jobsid: Data<Arc<Mutex<u32>>> = Data::new(Arc::new(Mutex::new(
        (match stmt.exists([]) {
            Ok(true) => stmt.query([])?.next()?.unwrap().get(0)?,
            _ => -1,
        } + 1) as u32)));
    // *jobsid.lock().await += 1;
    println!("{}", *jobsid.lock().await);

    log::info!("starting HTTP server at http://{}:{}", address, port);//config.server.bind_address, config.server.bind_port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(jobsid.clone())
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(prob_map.clone()))
            .app_data(Data::new(Mutex::new(pool.clone())))
            .configure(handler::route)
            .service(handler::exit)
            .default_service(route().to(handler::default_route))
    })
    .bind((address, port))?//(config.server.bind_address, config.server.bind_port))?
    .run()
    .await?;
    Ok(())
}
