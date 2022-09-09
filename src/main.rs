use actix_web::web::{self, route, Data};
use actix_web::{middleware::Logger, App, HttpServer};
use clap::Arg;
use config::Ids;
use env_logger;
use log;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod contests;
mod error_log;
mod handler;
mod runner;
mod users;

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
                .help("Toggle to flush OJ data in database."),
        )
        .get_matches();
    if matches.is_present("flush_data") {
        let _ = std::fs::remove_file("data.db");
    }
    let mut config_path: String = "config.json".to_string();
    if matches.is_present("config_path") {
        if let Some(path) = matches.value_of("config_path") {
            config_path = String::from(path);
        } else {
            panic!("No config path found.");
        }
    }

    let manager = SqliteConnectionManager::file("data.db");
    let pool = Pool::new(manager).unwrap();

    // Create a database table if not exists.
    let conn = pool.get().unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS jobs (id INT, created_time VARCHAR, updated_time VARCHAR, submission_id INT, state VARCHAR, result VARCHAR, score FLOAT, cases INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS submission (id INT, source_code VARCHAR, language VARCHAR, user_id INT, contest_id INT, problem_id INT);", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS cases (jobid INT, caseid INT, result VARCHAR, time INT, memory INT, info VARCHAR)", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR)",
        [],
    )?;
    conn.execute("CREATE TABLE IF NOT EXISTS contests (id INT, name VARCHAR, from_time VARCHAR, to_time VARCHAR, problem_ids VARCHAR, user_ids VARCHAR, submission_limit INT)", [])?;

    let config: config::Config =
        config::parse_from_file(config_path).expect("Config file format error.");
    let (address, port) = (
        config.server.bind_address.to_string(),
        config.server.bind_port,
    );

    let mut prob_map = HashMap::new();
    for prob in &config.problems {
        prob_map.insert(prob.id, prob.clone());
    }

    // get the maximal job, user and contest id
    let mut stmt = conn.prepare("SELECT * FROM jobs ORDER BY id DESC LIMIT 1;")?;
    let jobsid: i32 = match stmt.exists([]) {
        Ok(true) => stmt.query([])?.next()?.unwrap().get(0)?,
        _ => -1,
    } + 1;
    println!("Max Job ID: {}", jobsid);

    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY id DESC LIMIT 1;")?;
    let usersid: i32 = match stmt.exists([]) {
        Ok(true) => stmt.query([])?.next()?.unwrap().get(0)?,
        _ => -1,
    } + 1;
    println!("Max User ID: {}", usersid);
    let flag = usersid == 0;

    let mut stmt = conn.prepare("SELECT * FROM contests ORDER BY id DESC LIMIT 1;")?;
    let contestsid: i32 = match stmt.exists([]) {
        Ok(true) => stmt.query([])?.next()?.unwrap().get(0)?,
        _ => 0,
    } + 1;
    println!("Max Contest ID: {}", contestsid);

    let ids = Data::new(Arc::new(Mutex::new(Ids {
        jobsid: jobsid as u32,
        usersid: usersid as u32,
        contestsid: contestsid as u32,
    })));

    if flag {
        let _ = users::create_user(Data::new(Mutex::new(pool.clone())), "root", ids.clone()).await;
    }

    println!("{:?}", config);

    log::info!("starting HTTP server at http://{}:{}", address, port); //config.server.bind_address, config.server.bind_port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(ids.clone())
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(prob_map.clone()))
            .app_data(Data::new(Mutex::new(pool.clone())))
            .configure(handler::route)
            .service(handler::exit)
            .default_service(route().to(handler::default_route))
    })
    .bind((address, port))? //(config.server.bind_address, config.server.bind_port))?
    .run()
    .await?;
    Ok(())
}
