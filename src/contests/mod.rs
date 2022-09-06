use actix_web::web::Data;
use actix_web::{delete, get, post, put, web, Responder, HttpResponse, HttpResponseBuilder, HttpRequest};
use serde::{Deserialize, Serialize};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Result};
// use web::{Json, Path};
use tokio::sync::Mutex;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::prelude::*;

use crate::error_log;
use crate::config::{self, Config, Ids};
use crate::handler::jobs::{PostContest, ScoringRule, TieBreaker, RankFilter};
use crate::users::{self, SerdeUser};
use crate::runner::{self, SerdeJob};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SerdeContest {
    pub id: u32,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<u32>,
    pub user_ids: Vec<u32>,
    pub submission_limit: u32,
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SerdeRank {
    pub user: SerdeUser,
    pub rank: u32,
    pub scores: Vec<f32>,
}


#[derive(Debug, Default)]
pub struct TieBreakerStatus {
    pub submission_time: String,
    pub submission_count: u32,
    pub user_id: u32,
    pub score: f32,
}


pub async fn job_exists (pool: Data<Mutex<Pool<SqliteConnectionManager>>>, contest_id: u32) -> bool {
    let data = pool.lock().await.get().unwrap();
    let mut stmt = data.prepare(&format!("SELECT * FROM contests WHERE id = {};", contest_id)).expect("Database Error."); 
    match stmt.exists([]) {
        Ok(true) => true,
        _ => false,
    }
}


pub async fn get_contest(pool: Data<Mutex<Pool<SqliteConnectionManager>>>, contest_id: u32) -> Result<SerdeContest, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM contests WHERE id = :id;") {
        Ok(s) => stmt = s,
        _ => { return Err( error_log::EXTERNAL::webmsg("Database Error.")); }
    }
    if !stmt.exists(&[(":id", contest_id.to_string().as_str())]).unwrap() {
        return Err( error_log::NOT_FOUND::webmsg(&format!("Contest {} not found.", contest_id)));
    }
    let iter = stmt.query_map(&[(":id", contest_id.to_string().as_str())], |row| {
        Ok(SerdeContest {
            id: row.get(0)?,
            name: row.get(1)?,
            from: row.get(2)?,
            to: row.get(3)?,
            problem_ids: serde_json::from_str(&row.get::<_,String>(4)?).unwrap(),
            user_ids: serde_json::from_str(&row.get::<_,String>(5)?).unwrap(),
            submission_limit: row.get(6)?,
        })
    });
    match iter {
        Ok(mut ans) => { Ok(ans.next().unwrap().expect("Unknown Error.")) }
        _ => { Err( error_log::EXTERNAL::webmsg("Database Error.")) }
    }
}


pub async fn contest_exists (pool: Data<Mutex<Pool<SqliteConnectionManager>>>, contest_id: u32) -> bool {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM contests WHERE id = :id;") {
        Ok(s) => stmt = s,
        _ => { return true; }
    };
    stmt.exists(&[(":id", contest_id.to_string().as_str())]).unwrap()
}


pub async fn update_contest(body: web::Json<PostContest>, pool: Data<Mutex<Pool<SqliteConnectionManager>>>) -> HttpResponse {
    println!("Contests: Updating contest...");
    let mut contest: SerdeContest;
    match get_contest(pool.clone(), body.id.unwrap()).await {
        Ok(ans) => { contest = ans; }
        Err(e) => { return e; }
    };
    let data = pool.lock().await.get().unwrap();
    if let Err(e) = data.execute("UPDATE contests SET (name, from_time, to_time, problem_ids, user_ids, submission_limit) = (?1, ?2, ?3, ?4, ?5, ?6) WHERE id = ?7;", 
                            params![body.name, body.from, body.to, format!("{:?}", body.problem_ids).to_string(), format!("{:?}", body.user_ids).to_string(), body.submission_limit as i32, body.id.unwrap() as i32]) {
        return error_log::EXTERNAL::webmsg("Database Error.");
    }
    HttpResponse::Ok().body(serde_json::to_string_pretty(&contest).unwrap())
}


pub async fn create_contest(body: PostContest, pool: Data<Mutex<Pool<SqliteConnectionManager>>>, ids: Data<Arc<Mutex<Ids>>>) -> Result<SerdeContest, HttpResponse> {
    println!("Contests: Creating Contest...");

    let contest_id: u32 = ids.lock().await.contestsid;
    ids.lock().await.contestsid += 1;
    println!("contest ID: {}", contest_id);

    let data = pool.lock().await.get().unwrap();
    if let Err(e) = data.execute("INSERT INTO contests (id, name, from_time, to_time, problem_ids, user_ids, submission_limit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);", 
                            params![contest_id as i32, body.name, body.from, body.to, format!("{:?}", body.problem_ids).to_string(), format!("{:?}", body.user_ids).to_string(), body.submission_limit as i32]) {
        return Err( error_log::EXTERNAL::webmsg(&format!("Database Error: {}", e)));
    }

    Ok(SerdeContest{ id: contest_id, name: body.name, from: body.from, to: body.to, problem_ids: body.problem_ids, user_ids: body.user_ids, submission_limit: body.submission_limit})
}


pub async fn get_contests(pool: Data<Mutex<Pool<SqliteConnectionManager>>>) -> Result<Vec<SerdeContest>, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM contests ORDER BY id;") {
        Ok(s) => stmt = s,
        _ => { return Err( error_log::EXTERNAL::webmsg("Database Error.")); }
    }
    let iter = stmt.query_map([],|row| {
        Ok(SerdeContest {
            id: row.get(0)?,
            name: row.get(1)?,
            from: row.get(2)?,
            to: row.get(3)?,
            problem_ids: serde_json::from_str(&row.get::<_,String>(4)?).unwrap(),
            user_ids: serde_json::from_str(&row.get::<_,String>(5)?).unwrap(),
            submission_limit: row.get(6)?,
        })
    }).expect("Unknown Error.");
    let mut contests: Vec<SerdeContest> = vec![];
    for user in iter {
        contests.push(user.unwrap());
    }
    Ok(contests)
}

pub fn eq(s1: &TieBreakerStatus, s2: &TieBreakerStatus, filter: &RankFilter) -> bool {
    if s1.score.ne(&s2.score) { return false; }
    match filter.tie_breaker {
        TieBreaker::submission_time => s1.submission_time.eq(&s2.submission_time),
        TieBreaker::submission_count => s1.submission_count.eq(&s2.submission_count),
        TieBreaker::user_id => s1.user_id.eq(&s2.user_id),
        _ => true,
    }
}

pub async fn get_ranklist(pool: Data<Mutex<Pool<SqliteConnectionManager>>>, config: Data<Config>, mut filter: RankFilter, contest_id: u32, ids: Data<Arc<Mutex<Ids>>>) -> Result<Vec<SerdeRank>, HttpResponse> {
    println!("Getting ranklist...");
    let mut ans: Vec<SerdeJob> = vec![];
    let mut contest: SerdeContest = Default::default();
    if contest_id != 0 {
        if let Ok(con) = get_contest(pool.clone(), contest_id).await {
            contest = con;
        } else { return Err( error_log::NOT_FOUND::webmsg(&format!("Contest {} not found.", contest_id))); }
    } else {
        for prob in &config.problems {
            contest.problem_ids.push(prob.id);
        }
    }

    let zero_time = "0001-01-01T00:00:01.000Z".to_string();
    let inf_time = "9999-12-31T23:59:59.000Z".to_string();
    let mut user_score: HashMap<u32, Vec<f32>> = HashMap::new();
    let mut prob_id_map: HashMap<u32, u32> = HashMap::new();
    let mut user_id_map: HashMap<u32, usize> = HashMap::new();
    let prob_tot = contest.problem_ids.len();

    let mut tbstatus: Vec<TieBreakerStatus> = vec![];
    for (index, prob_id) in contest.problem_ids.iter().enumerate() {
        prob_id_map.insert(*prob_id, index as u32);
    }
    let tot = ids.lock().await.jobsid as i32;
    let user_tot = ids.lock().await.usersid;
    for id in 0..tot {
        let job;
        match runner::get_a_job(pool.clone(), id as u32).await {
            Ok(ans) => { job = ans; },
            Err(e) => { return Err(e); }
        }
        let prob_id = job.submission.problem_id;
        let user_id = job.submission.user_id;
        println!("prob{} user{}", prob_id, user_id);
        println!("{:?}", contest);
        if !contest.problem_ids.contains(&prob_id) { continue; }
        if contest_id != 0 && !contest.user_ids.contains(&user_id) { continue; }
        if contest_id != 0 && contest.id != job.submission.contest_id { continue; }
        if !job.state.eq("Finished") { continue; }
        println!("cur: {} , get {:?}", contest_id, job);
        let prob_index = *prob_id_map.get(&prob_id).unwrap();
        if !user_score.contains_key(&user_id) {
            user_score.insert(user_id, vec![0.0; prob_tot]);
            tbstatus.push(TieBreakerStatus{ submission_time: zero_time.clone(), submission_count: 0, user_id: user_id, score: 0.0 });
            user_id_map.insert(user_id, tbstatus.len() - 1);
        }
        let mut status = &mut tbstatus[*user_id_map.get(&user_id).unwrap()];
        status.submission_count += 1;
        match filter.scoring_rule {
            ScoringRule::highest => {
                let score = user_score.get(&user_id).unwrap()[prob_index as usize];
                if job.score > score {
                    user_score.get_mut(&user_id).unwrap()[prob_index as usize] = job.score;
                    status.submission_time = String::from(&job.created_time);
                }
            },
            ScoringRule::latest => {
                user_score.get_mut(&user_id).unwrap()[prob_index as usize] = job.score;
                status.submission_time = String::from(&job.created_time);
            }
        };
    }

    //Sort for ranklist
    for status in &mut tbstatus {
        if status.submission_time.eq(&zero_time) {
            status.submission_time = String::from(&inf_time);
        }
        for score in &user_score[&status.user_id] {
            status.score += score;
        }
        println!("{:?}", status);
    }
    if contest_id != 0 {
        for user_id in contest.user_ids {
            if !user_score.contains_key(&user_id) {
                user_score.insert(user_id, vec![0.0; prob_tot]);
                tbstatus.push(TieBreakerStatus{ submission_time: inf_time.clone(), submission_count: 0, user_id: user_id, score: 0.0 });
            }
        }
    } else {
        for user_id in 0..user_tot {
            if !user_score.contains_key(&user_id) {
                user_score.insert(user_id, vec![0.0; prob_tot]);
                tbstatus.push(TieBreakerStatus{ submission_time: inf_time.clone(), submission_count: 0, user_id: user_id, score: 0.0 });
            }
        }
    }
    tbstatus.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    match filter.tie_breaker {
        TieBreaker::submission_time => {
            tbstatus.sort_by(|a, b| a.submission_time.cmp(&b.submission_time));
        },
        TieBreaker::submission_count => {
            tbstatus.sort_by(|a, b| a.submission_count.cmp(&b.submission_count));
        },
        _ => {},
    }
    tbstatus.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let mut ans: Vec<SerdeRank> = vec![];
    let mut rank: u32 = 0;
    let mut las: &TieBreakerStatus = &Default::default();
    for (index, status) in tbstatus.iter().enumerate() {
        let user;
        match users::get_user(pool.clone(), status.user_id).await {
            Ok(ans) => { user = ans; },
            Err(e) => {return Err(e); },
        }
        if index == 0 || (index != 0 && !eq(&status, las, &filter)) { rank = (index as u32) + 1; }
        let mut cur = SerdeRank{ user: user, rank: rank, scores: vec![] };
        for score in user_score.get(&status.user_id).unwrap() {
            cur.scores.push(*score);
        }
        las = &status;
        ans.push(cur);
    }

    Ok(ans)
}
