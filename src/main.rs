use log;
use env_logger;
use rusqlite::{params, Connection};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::result::Result as StdResult;
use actix_web::{middleware::Logger, App, HttpServer, Responder};
use actix_web::web::{self, route, Data};
use std::collections::HashMap;
use clap::{Arg, ArgMatches};

mod handler;
mod config;
mod error_log;
mod runner;

pub type Result<T = (), E = Box<dyn Error>> = StdResult<T, E>;

#[actix_web::main]
async fn main() -> Result {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Get args
    let matches = clap::App::new("mroj")
        .version("0.1.0")
        .author("Jashng")
        .about("An online judge backend.")
        .arg(
            Arg::with_name("config_path")
                .short('c')
                .long("config")
                .takes_value(true)
                .help("The config file path."),
        )
        .arg(
            Arg::with_name("flush_data")
                .short('f')
                .long("flush-data")
                .takes_value(false)
                .help("Toggle to flush OJ data in database.")
        )
        .get_matches();
    if matches.is_present("flush_data") {
        let _ = std::fs::remove_file("data.db");
    }
    let mut config_path: String = "config.json".to_string();
    if matches.is_present("config_path") {
        if let Some(path) = matches.value_of("config_path") {
            config_path = String::from(path);
        } else { panic!("No config path found."); }
    }

    let manager = SqliteConnectionManager::file("data.db");
    let pool = Pool::new(manager).unwrap();

    // Create a database table if not exists.
    let conn = pool.get().unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS jobs (id INT, created_time VARCHAR, updated_time VARCHAR, submission_id INT, state VARCHAR, result VARCHAR, score FLOAT, cases INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS submission (id INT, source_code VARCHAR, language VARCHAR, user_id INT, contest_id INT, problem_id INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS cases (jobid INT, caseid INT, result VARCHAR, time INT, memory INT, info VARCHAR)", [])?;

    let config: config::Config = config::parse_from_file(config_path).expect("Config file format error.");
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

    println!("{:?}", config);

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
