use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use chrono::prelude::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use wait_timeout::ChildExt;

use crate::config::{self, Config, Ids, ProbType, Problem};
use crate::handler::jobs::{JobsFilter, PostJob};
use crate::{error_log, users};

mod diff;

#[derive(Debug, Deserialize, Serialize)]
pub struct SerdeJob {
    pub id: u32,
    pub created_time: String,
    pub updated_time: String,
    pub submission: SerdeSubmission,
    pub state: String,
    pub result: String,
    pub score: f32,
    cases: Vec<SerdeCase>,
}

impl SerdeJob {
    pub fn get_post(&self) -> PostJob {
        return PostJob {
            source_code: self.submission.source_code.to_string(),
            language: self.submission.language.to_string(),
            user_id: self.submission.user_id,
            contest_id: self.submission.contest_id,
            problem_id: self.submission.problem_id,
        };
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct SerdeSubmission {
    pub source_code: String,
    pub language: String,
    pub user_id: u32,
    pub contest_id: u32,
    pub problem_id: u32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct SerdeCase {
    id: u32,
    result: String,
    time: u32,
    memory: u32,
    info: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Job {
    id: u32,
    created_time: String,
    updated_time: String,
    submission: u32,
    state: String,
    result: String,
    score: f32,
    cases: Vec<String>,
}

impl Job {
    fn new(id: u32, sub_id: u32) -> Job {
        let time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        Job {
            id: id,
            created_time: time.to_string(),
            updated_time: time.to_string(),
            submission: sub_id,
            state: "Queueing".to_string(),
            result: "Waiting".to_string(),
            score: 0.0,
            cases: vec![],
        }
    }
}

pub async fn job_exists(pool: Data<Mutex<Pool<SqliteConnectionManager>>>, job_id: u32) -> bool {
    let data = pool.lock().await.get().unwrap();
    let mut stmt = data
        .prepare(&format!("SELECT * FROM jobs WHERE id = {};", job_id))
        .expect("Database Error.");
    match stmt.exists([]) {
        Ok(true) => true,
        _ => false,
    }
}

pub async fn get_submission(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    job_id: u32,
) -> Result<SerdeSubmission, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut sub_stmt;
    match data.prepare("SELECT * FROM submission WHERE id = :id;") {
        Ok(stmt) => sub_stmt = stmt,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    if !sub_stmt
        .exists(&[(":id", job_id.to_string().as_str())])
        .unwrap()
    {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "Job {} not found.",
            job_id
        )));
    }
    let sub_iter = sub_stmt.query_map(&[(":id", job_id.to_string().as_str())], |row| {
        Ok(SerdeSubmission {
            source_code: row.get(1)?,
            language: row.get(2)?,
            user_id: row.get(3)?,
            contest_id: row.get(4)?,
            problem_id: row.get(5)?,
        })
    });
    match sub_iter {
        Ok(mut ans) => Ok(ans.next().unwrap().expect("Unknown Error.")),
        _ => Err(error_log::EXTERNAL::webmsg("Database Error.")),
    }
}

pub async fn get_a_job(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    job_id: u32,
) -> Result<SerdeJob, HttpResponse> {
    // get submission
    let submission;
    match get_submission(pool.clone(), job_id).await {
        Ok(sub) => {
            submission = sub;
        }
        Err(e) => {
            return Err(e);
        }
    }

    //get cases
    let data = pool.lock().await.get().unwrap();
    let mut cases_stmt;
    match data.prepare("SELECT * FROM cases WHERE jobid = :id ORDER BY caseid;") {
        Ok(stmt) => cases_stmt = stmt,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    if !cases_stmt
        .exists(&[(":id", job_id.to_string().as_str())])
        .unwrap()
    {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "Job {} not found.",
            job_id
        )));
    }
    let cases_iter = cases_stmt
        .query_map(&[(":id", job_id.to_string().as_str())], |row| {
            Ok(SerdeCase {
                id: row.get(1)?,
                result: row.get(2)?,
                time: row.get(3)?,
                memory: row.get(4)?,
                info: row.get(5)?,
            })
        })
        .expect("Unknown Error.");
    let mut cases: Vec<SerdeCase> = vec![];
    for case in cases_iter {
        cases.push(case.unwrap());
    }

    // get job
    let mut job_stmt;
    match data.prepare("SELECT * FROM jobs WHERE id = :id;") {
        Ok(stmt) => job_stmt = stmt,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    if !job_stmt
        .exists(&[(":id", job_id.to_string().as_str())])
        .unwrap()
    {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "Job {} not found.",
            job_id
        )));
    }
    let mut job_iter = job_stmt
        .query_map(&[(":id", job_id.to_string().as_str())], |row| {
            Ok(SerdeJob {
                id: row.get(0)?,
                created_time: row.get(1)?,
                updated_time: row.get(2)?,
                submission: SerdeSubmission {
                    ..SerdeSubmission::default()
                },
                state: row.get(4)?,
                result: row.get(5)?,
                score: row.get(6)?,
                cases: vec![],
            })
        })
        .expect("Unknown Error.");
    let mut job: SerdeJob = job_iter.next().unwrap().expect("Unknown Error.");
    job.submission = submission;
    job.cases = cases;

    println!("Job: {:?}", job);

    Ok(job)
}

pub async fn get_job(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    job_id: u32,
) -> HttpResponse {
    match get_a_job(pool.clone(), job_id).await {
        Ok(job) => HttpResponse::Ok().body(serde_json::to_string_pretty(&job).unwrap()),
        Err(e) => e,
    }
}

pub async fn get_jobs(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    mut filter: JobsFilter,
    ids: Data<Arc<Mutex<Ids>>>,
) -> Result<Vec<SerdeJob>, HttpResponse> {
    let mut ans: Vec<SerdeJob> = vec![];
    if let Some(name) = filter.user_name {
        if let Ok(id) = users::get_user_id(pool.clone(), &name).await {
            if let Some(user_id) = filter.user_id {
                if id != user_id {
                    return Err(HttpResponse::Ok().body("[]"));
                }
            } else {
                filter.user_id = Some(id);
            }
        }
    }
    let tot = ids.lock().await.jobsid as i32;
    for id in 0..tot {
        let job = get_a_job(pool.clone(), id as u32)
            .await
            .expect("Get Job Error.");
        if let Some(user_id) = filter.user_id {
            if job.submission.user_id != user_id {
                continue;
            }
        }
        //TODO: user_name
        if let Some(contest_id) = filter.contest_id {
            if job.submission.contest_id != contest_id {
                continue;
            }
        }
        if let Some(problem_id) = filter.problem_id {
            if job.submission.problem_id != problem_id {
                continue;
            }
        }
        if let Some(language) = &filter.language {
            println!("{}", language);
            if !job.submission.language.eq(language) {
                continue;
            }
        }
        if let Some(from) = &filter.from {
            if job.created_time.cmp(from) == Ordering::Less {
                continue;
            }
        }
        if let Some(to) = &filter.to {
            if job.created_time.cmp(to) == Ordering::Greater {
                continue;
            }
        }
        if let Some(state) = &filter.state {
            if !job.state.eq(state) {
                continue;
            }
        }
        if let Some(result) = &filter.result {
            if !job.result.eq(result) {
                continue;
            }
        }
        ans.push(job);
    }
    Ok(ans)
}

pub async fn reset_job(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    job_id: u32,
    prob_map: Data<HashMap<u32, config::Problem>>,
) -> Result<(), HttpResponse> {
    if job_exists(pool.clone(), job_id).await {
        let data = pool.lock().await.get().unwrap();
        let mut stmt = data
            .prepare(&format!("SELECT * FROM jobs WHERE id = {};", job_id))
            .expect("Database Error.");
        if !stmt
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .get::<_, String>(4)
            .unwrap()
            .eq("Finished")
        {
            return Err(error_log::INVALID_STATE::webmsg(&format!(
                "Job {} not finished.",
                job_id
            )));
        }
        drop(stmt);
        drop(data);
    } else {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "Job {} not found.",
            job_id
        )));
    }

    let data = pool.lock().await.get().unwrap();
    let prob_id;
    match get_submission(pool.clone(), job_id).await {
        Ok(sub) => {
            prob_id = sub.problem_id;
        }
        Err(e) => {
            return Err(e);
        }
    }

    let top = prob_map.get(&prob_id).unwrap().cases.len();

    let time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let _ = data.execute(
        "UPDATE jobs SET updated_time = ?1 WHERE id = ?2",
        params![time.to_string(), job_id],
    );
    let _ = data.execute(
        "UPDATE jobs SET state = 'Queueing' WHERE id = ?1",
        params![job_id],
    );
    let _ = data.execute(
        "UPDATE jobs SET result = 'Waiting' WHERE id = ?1",
        params![job_id],
    );
    let _ = data.execute("UPDATE jobs SET score = 0.0 WHERE id = ?1", params![job_id]);

    for i in 0..top {
        let _ = data.execute(
            "UPDATE cases SET result = 'Waiting' WHERE jobid = ?1 AND caseid = ?2",
            params![job_id, i as i32],
        );
        let _ = data.execute(
            "UPDATE cases SET time = 0 WHERE jobid = ?1 AND caseid = ?2",
            params![job_id, i as i32],
        );
        let _ = data.execute(
            "UPDATE cases SET memory = 0 WHERE jobid = ?1 AND caseid = ?2",
            params![job_id, i as i32],
        );
        let _ = data.execute(
            "UPDATE cases SET info = '' WHERE jobid = ?1 AND caseid = ?2",
            params![job_id, i as i32],
        );
    }

    Ok(())
}

async fn create_task(
    body: web::Json<PostJob>,
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    prob_map_shared: Data<HashMap<u32, Problem>>,
    ids: Data<Arc<Mutex<Ids>>>,
) -> (HttpResponse, u32) {
    println!("Runner: Creating Job...");

    let data = pool.lock().await.get().unwrap();

    let job_id: u32 = ids.lock().await.jobsid;
    ids.lock().await.jobsid += 1;
    println!("Job ID: {}", job_id);

    match data.execute("INSERT INTO submission (id, source_code, language, user_id, contest_id, problem_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
        params![job_id as i32, body.source_code, body.language, body.user_id as i32, body.contest_id as i32, body.problem_id as i32]) {
            Err(_) => { return (error_log::EXTERNAL::webmsg("Database Error."), job_id); },
            _ => {},
        };
    let prob = prob_map_shared.get(&body.problem_id).unwrap();
    for index in 0..=prob.cases.len() {
        match data.execute("INSERT INTO cases (jobid, caseid, result, time, memory, info) VALUES (?1, ?2, 'Waiting', 0, 0, '');", 
            params![job_id, index]) {
            Err(_) => { return (error_log::EXTERNAL::webmsg("Database Error."), job_id); }
            _ => {},
        };
    }
    let mut cur = Job::new(job_id, job_id);
    for i in 0..=prob.cases.len() {
        cur.cases.push(format!("{}-{}", job_id, i).to_string());
    }
    match data.execute("INSERT INTO jobs (id, created_time, updated_time, submission_id, state, result, score, cases) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![job_id as i32, cur.created_time, cur.updated_time, cur.submission as i32, cur.state, cur.result, 0.0 as f32, cur.cases.len() as i32 - 1]) {
            Err(_) => { return (error_log::EXTERNAL::webmsg("Database Error."), job_id); },
            _ => {},
        }
    (get_job(pool, job_id).await, job_id)
}

pub async fn run(
    body: PostJob,
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    config: Data<Config>,
    prob_map: Data<HashMap<u32, Problem>>,
    job_id: u32,
) {
    let _ = std::fs::create_dir("oj_runtime_dir");
    let _ = std::fs::remove_dir_all(format!("oj_runtime_dir/job_{}", job_id));
    let _ = std::fs::create_dir(format!("oj_runtime_dir/job_{}", job_id));
    let path = format!("oj_runtime_dir/job_{}", job_id).to_string();
    let mut lang = config::Language {
        ..config::Language::default()
    };
    for language in &config.languages {
        if language.name.eq(&body.language) {
            lang.name = language.name.to_string();
            lang.file_name = language.file_name.to_string();
            lang.command = language.command.iter().map(|s| s.to_string()).collect();
            break;
        }
    }
    println!("Language: {:?}", lang);
    let mut file =
        std::fs::File::create(format!("{}/{}", path, lang.file_name)).expect("Cannot create file.");
    let _ = file.write_all(body.source_code.as_bytes());

    // Compilation Part
    let (mut input_index, mut output_index): (Option<usize>, Option<usize>) = (None, None);
    for (index, arg) in lang.command.iter().enumerate() {
        if arg.eq("%INPUT%") {
            input_index = Some(index);
        } else if arg.eq("%OUTPUT%") {
            output_index = Some(index);
        }
    }
    let bin_path: String = match cfg!(target_os = "windows") {
        true => format!("{}/job.exe", path).to_string(),
        false => format!("{}/job", path).to_string(),
    };

    if input_index.is_some() {
        lang.command[input_index.unwrap()] = format!("{}/{}", path, lang.file_name);
    }
    if output_index.is_some() {
        lang.command[output_index.unwrap()] = bin_path.to_string();
    }

    // Start compiling
    let data = pool.lock().await.get().unwrap();
    let _ = data.execute(
        "UPDATE jobs SET state = 'Running' WHERE id = ?1;",
        params![job_id as i32],
    );
    let _ = data.execute(
        "UPDATE jobs SET result = 'Running' WHERE id = ?1;",
        params![job_id as i32],
    );
    let _ = data.execute(
        "UPDATE cases SET result = 'Running' WHERE jobid = ?1 AND caseid = ?2;",
        params![job_id as i32, 0],
    );
    drop(data);
    let mut compiler = Command::new(&lang.command[0])
        .args(&lang.command[1..])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let wait_time = Duration::from_secs(15); //compiling for at most 15 seconds
    let status_code = match compiler.wait_timeout(wait_time).unwrap() {
        Some(status) => status.code(),
        None => {
            compiler.kill().unwrap();
            compiler.wait().unwrap().code()
        }
    };

    //Compilation finished
    let data = pool.lock().await.get().unwrap();
    match status_code {
        Some(0) => {
            //Compilation Success
            let _ = data.execute(
                "UPDATE cases SET result = 'Compilation Success' WHERE jobid = ?1 AND caseid = ?2;",
                params![job_id as i32, 0],
            );
        }
        _ => {
            let _ = data.execute(
                "UPDATE jobs SET state = 'Finished' WHERE id = ?1;",
                params![job_id as i32],
            );
            let _ = data.execute(
                "UPDATE jobs SET result = 'Compilation Error' WHERE id = ?1;",
                params![job_id as i32],
            );
            let _ = data.execute(
                "UPDATE cases SET result = 'Compilation Error' WHERE jobid = ?1 AND caseid = ?2;",
                params![job_id as i32, 0],
            );
            return;
        }
    }
    drop(data);

    // Running Cases Part
    let cases = &prob_map.get(&body.problem_id).unwrap().cases;
    let mut score: f32 = 0.0;
    let mut flag: bool = true;
    let mut indexes: Vec<Vec<u32>> = vec![];
    match &prob_map.get(&body.problem_id).unwrap().misc.packing {
        Some(pack) => {
            for index in pack {
                let mut tmp_vec = vec![];
                for i in index {
                    tmp_vec.push(i - 1);
                }
                indexes.push(tmp_vec);
            }
        }
        None => {
            let tot = cases.len();
            for i in 0..tot {
                indexes.push(vec![i as u32]);
            }
        }
    };
    println!("{:?}", cases);
    for index0 in &indexes {
        let mut skip_flag = false;
        let mut pack_score: f32 = 0.0;
        for index_tmp in index0 {
            let case = &cases[*index_tmp as usize];
            let index = *index_tmp as i32 + 1;
            // Check if skipped
            if skip_flag {
                let data = pool.lock().await.get().unwrap();
                let _ = data.execute(
                    "UPDATE cases SET result = 'Skipped' WHERE jobid = ?1 AND caseid = ?2;",
                    params![job_id as i32, index as i32],
                );
                drop(data);
                continue;
            } else {
                skip_flag = true;
            }

            // Update database
            let data = pool.lock().await.get().unwrap();
            let _ = data.execute(
                "UPDATE cases SET result = 'Running' WHERE jobid = ?1 AND caseid = ?2;",
                params![job_id as i32, index as i32],
            );
            drop(data);

            // Running
            let out_file = format!("{}/{}.out", path, index).to_string();
            let now = Instant::now();
            let mut runner = Command::new(&bin_path)
                .stdin(Stdio::from(std::fs::File::open(&case.input_file).unwrap()))
                .stdout(Stdio::from(std::fs::File::create(&out_file).unwrap()))
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let wait_time = Duration::from_micros(case.time_limit);
            let mut real_time: u128 = 0;
            match runner.wait_timeout(wait_time).unwrap() {
                Some(status) => {
                    if status.code().unwrap() != 0 {
                        //Runtime Error
                        let data = pool.lock().await.get().unwrap();
                        let _ = data.execute("UPDATE cases SET result = 'Runtime Error' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                        if flag {
                            let _ = data.execute(
                                "UPDATE jobs SET result = 'Runtime Error' WHERE id = ?1;",
                                params![job_id as i32],
                            );
                            flag = false;
                        }
                        drop(data);
                        continue;
                    } else {
                        //Exited Normally
                        real_time = now.elapsed().as_micros();
                    }
                }
                None => {
                    //Time Limit Exceeded
                    real_time = now.elapsed().as_micros();
                    let data = pool.lock().await.get().unwrap();
                    let _ = data.execute("UPDATE cases SET result = 'Time Limit Exceeded' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                    let _ = data.execute(
                        "UPDATE cases SET time = ?1 WHERE jobid = ?2 AND caseid = ?3;",
                        params![real_time as i32, job_id as i32, index as i32],
                    );
                    if flag {
                        let _ = data.execute(
                            "UPDATE jobs SET result = 'Time Limit Exceeded' WHERE id = ?1;",
                            params![job_id as i32],
                        );
                        flag = false;
                    }
                    drop(data);
                    continue;
                }
            };

            // Exited Normally
            let diff_code = match prob_map.get(&body.problem_id).unwrap().ty {
                ProbType::standard => diff::diff_standard(&case.answer_file, &out_file),
                ProbType::strict => diff::diff_strict(&case.answer_file, &out_file),
                ProbType::spj => {
                    let mut spj_info: Vec<String> = vec![];
                    match &prob_map.get(&body.problem_id).unwrap().misc.special_judge {
                        Some(info) => {
                            spj_info = info.to_vec();
                        }
                        None => {
                            let data = pool.lock().await.get().unwrap();
                            let _ = data.execute("UPDATE cases SET result = 'SPJ Error' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                            let _ = data.execute("UPDATE cases SET info = 'No SPJ specified in config: misc' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                            drop(data);
                            continue;
                        }
                    }
                    for i in 0..spj_info.len() {
                        if spj_info[i].eq("%OUTPUT%") {
                            // out_index = Some()
                            spj_info[i] = out_file.to_string();
                        } else if spj_info[i].eq("%ANSWER%") {
                            spj_info[i] = case.answer_file.to_string();
                        }
                    }
                    let ans: usize;
                    match diff::diff_spj(&spj_info).await {
                        Ok(info) => {
                            ans = info.0;
                            let data = pool.lock().await.get().unwrap();
                            let _ = data.execute(
                                "UPDATE cases SET info = ?1 WHERE jobid = ?2 AND caseid = ?3;",
                                params![info.1, job_id as i32, index as i32],
                            );
                            drop(data);
                        }
                        Err(_) => {
                            let data = pool.lock().await.get().unwrap();
                            let _ = data.execute("UPDATE cases SET result = 'SPJ Error' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                            let _ = data.execute("UPDATE cases SET info = 'No SPJ specified in config: misc' WHERE jobid = ?1 AND caseid = ?2;", params![job_id as i32, index as i32]);
                            drop(data);
                            continue;
                        }
                    };
                    ans
                }
                _ => 0,
            };
            if diff_code == 0 {
                // Accepted
                pack_score += case.score;
                let data = pool.lock().await.get().unwrap();
                let _ = data.execute(
                    "UPDATE cases SET result = 'Accepted' WHERE jobid = ?1 AND caseid = ?2;",
                    params![job_id as i32, index as i32],
                );
                let _ = data.execute(
                    "UPDATE cases SET time = ?1 WHERE jobid = ?2 AND caseid = ?3;",
                    params![real_time as i32, job_id as i32, index as i32],
                );
                drop(data);
                skip_flag = false;
            } else {
                // Wrong Answer
                let data = pool.lock().await.get().unwrap();
                let _ = data.execute(
                    "UPDATE cases SET result = 'Wrong Answer' WHERE jobid = ?1 AND caseid = ?2;",
                    params![job_id as i32, index as i32],
                );
                if flag {
                    let _ = data.execute(
                        "UPDATE jobs SET result = 'Wrong Answer' WHERE id = ?1;",
                        params![job_id as i32],
                    );
                    flag = false;
                }
                drop(data);
            }
        }
        if !skip_flag {
            score += pack_score;
            let data = pool.lock().await.get().unwrap();
            let _ = data.execute(
                "UPDATE jobs SET score = ?1 WHERE id = ?2;",
                params![score, job_id as i32],
            );
            drop(data);
        }
    }

    //Finished
    let data = pool.lock().await.get().unwrap();
    let _ = data.execute(
        "UPDATE jobs SET state = 'Finished' WHERE id = ?1;",
        params![job_id as i32],
    );
    if flag {
        let _ = data.execute(
            "UPDATE jobs SET result = 'Accepted' WHERE id = ?1;",
            params![job_id as i32],
        );
    }
}

pub async fn start(
    body: web::Json<PostJob>,
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    config: Data<Config>,
    prob_map: Data<HashMap<u32, Problem>>,
    ids: Data<Arc<Mutex<Ids>>>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let pool_shared = pool.clone();
    let prob_map_shared = prob_map.clone();
    let post_job: PostJob = PostJob {
        source_code: body.source_code.to_string(),
        language: body.language.to_string(),
        user_id: body.user_id,
        contest_id: body.contest_id,
        problem_id: body.problem_id,
    };
    let (ans, job_id) = create_task(body, pool_shared, prob_map_shared, ids.clone()).await;
    let _ = tokio::spawn(async move {
        run(
            post_job,
            pool.clone(),
            config.clone(),
            prob_map.clone(),
            job_id,
        )
        .await;
    }); //.await;
    Ok(ans)
}
