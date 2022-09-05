use actix_web::web::Data;
use actix_web::{delete, get, post, put, web, Responder, HttpResponse, HttpResponseBuilder, HttpRequest};
use serde::{Deserialize, Serialize};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Result};
// use web::{Json, Path};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use crate::error_log;
use crate::config::{self, Config, Ids};
use crate::runner::{self, SerdeJob};
use crate::users::{self, SerdeUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostJob {
    pub source_code: String,
    pub language: String,
    pub user_id: u32,
    pub contest_id: u32,
    pub problem_id: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PostUser {
    pub id: Option<u32>,
    pub name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Filter {
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
    pub contest_id: Option<u32>,
    pub problem_id: Option<u32>,
    pub language: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub state: Option<String>,
    pub result: Option<String>,
}

#[post("/jobs")]
pub async fn post_job(body: web::Json<PostJob>, pool: Data<Mutex<Pool<SqliteConnectionManager>>>, config: Data<Config>, prob_map: Data<HashMap<u32, config::Problem>>, 
    ids: Data<Arc<Mutex<Ids>>>) -> HttpResponse {

    // check request
    if !config.languages.iter().map(|x| x.name.to_string()).collect::<Vec<String>>().contains(&body.language) {
        return HttpResponse::NotFound().body(error_log::NOT_FOUND::msg(&format!("Language {} no found.", body.language)));
    }
    if !config.problems.iter().map(|x| x.id).collect::<Vec<u32>>().contains(&body.problem_id) {
        return HttpResponse::NotFound().body(error_log::NOT_FOUND::msg(&format!("Problem with id({}) not found.", body.problem_id)));
    }
    if let Err(_) = users::get_user(pool.clone(), body.user_id).await {
        return HttpResponse::NotFound().body(error_log::NOT_FOUND::msg(&format!("User with id({}) not found.", body.user_id)));
    }
    // TODO: submission limit check
    // TODO: check contest_id

    runner::start(body, pool, config, prob_map, ids.clone()).await.unwrap()
}


#[get("/jobs/{jobid}")]
pub async fn get_job_by_id(path: web::Path<String>, pool: Data<Mutex<Pool<SqliteConnectionManager>>>) -> HttpResponse {
    let mut job_id: u32 = 0;
    match path.parse::<u32>() {
        Ok(id) => job_id = id,
        _ => { return error_log::NOT_FOUND::webmsg(&format!("Job {} not found.", path)); }
    };
    runner::get_job(pool, job_id).await
}


#[get("/jobs")]
pub async fn get_jobs(req: HttpRequest, pool: Data<Mutex<Pool<SqliteConnectionManager>>>, ids: Data<Arc<Mutex<Ids>>>) -> HttpResponse {
    let mut filter;
    let reqstr = str::replace(req.query_string(), "+", "🜔");
    println!("{:?}", reqstr);
    
    match web::Query::<Filter>::from_query(&reqstr) {
        Ok(flt) => filter = flt,
        _ => { return error_log::INVALID_ARGUMENT::webmsg("Invalid argument."); },
    };
    if let Some(language) = &filter.language {
        filter.language = Some(str::replace(language, "🜔", "+"));
    }

    runner::get_jobs(pool, filter.into_inner(), ids).await
}


#[put("/jobs/{jobid}")]
pub async fn rejudge_job_by_id(path: web::Path<String>, pool: Data<Mutex<Pool<SqliteConnectionManager>>>, ids: Data<Arc<Mutex<Ids>>>, config: Data<Config>, prob_map: Data<HashMap<u32, config::Problem>>) 
    -> HttpResponse {
    println!("Rejuding...");
    let mut job_id: u32 = 0;
    match path.parse::<u32>() {
        Ok(id) => job_id = id,
        _ => { return error_log::NOT_FOUND::webmsg(&format!("Job {} not found.", path)); }
    };
    if job_id >= ids.lock().await.jobsid { return error_log::NOT_FOUND::webmsg(&format!("Job {} not found.", path)); }
    match runner::reset_job(pool.clone(), job_id, prob_map.clone()).await {
        Err(e) => { return e; },
        _ => {},
    }
    let res = runner::get_a_job(pool.clone(), job_id).await;
    let ans;
    let post;
    match res {
        Ok(job) => { 
            post = job.get_post(); 
            ans = HttpResponse::Ok().body(serde_json::to_string_pretty(&job).unwrap()); 
        }
        Err(e) => { return e; },
    }
    let _ = tokio::spawn(async move {
        runner::run(post, pool.clone(), config.clone(), prob_map.clone(), job_id).await;
    });//.await;
    ans
}

#[post("/users")]
pub async fn post_user(body: web::Json<PostUser>, pool: Data<Mutex<Pool<SqliteConnectionManager>>>, ids: Data<Arc<Mutex<Ids>>>) -> HttpResponse {
    if let Some(id) = body.id {
        users::update_user(pool, id, &body.name).await
    } else {
        match users::create_user(pool, &body.name, ids.clone()).await {
            Ok(user) => HttpResponse::Ok().body(serde_json::to_string_pretty(&user.0).unwrap()),
            Err(e) => e,
        }
    }
}

#[get("/users")]
pub async fn get_users(pool: Data<Mutex<Pool<SqliteConnectionManager>>>) -> HttpResponse {
    match users::get_users(pool).await {
        Ok(users) => HttpResponse::Ok().body(serde_json::to_string_pretty(&users).unwrap()),
        Err(e) => e,
    }
}