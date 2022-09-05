use actix_web::HttpServer;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::{delete, post, get, web};
use actix_web::Responder;
use serde::{Deserialize, Serialize};

pub mod jobs;


#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}


pub fn route(config: &mut web::ServiceConfig) {
    config.service(jobs::post_job);
    config.service(jobs::get_job_by_id);
    config.service(jobs::get_jobs);
    config.service(jobs::rejudge_job_by_id);
    config.service(jobs::post_user);
    config.service(jobs::get_users);
}


pub async fn default_route() -> impl Responder {
    r#"{"code":"3","message":"404 Not Found"}"#
}
