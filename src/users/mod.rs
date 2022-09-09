use actix_web::web::Data;
use actix_web::HttpResponse;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
// use web::{Json, Path};
use crate::config::Ids;
use crate::error_log;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct SerdeUser {
    id: u32,
    name: String,
}

pub async fn user_name_exists(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    user_name: &str,
) -> bool {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM users WHERE name = :name;") {
        Ok(s) => stmt = s,
        _ => {
            return true;
        }
    };
    stmt.exists(&[(":name", user_name)]).unwrap()
}

pub async fn user_exists(pool: Data<Mutex<Pool<SqliteConnectionManager>>>, user_id: u32) -> bool {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM users WHERE id = :id;") {
        Ok(s) => stmt = s,
        _ => {
            return true;
        }
    };
    stmt.exists(&[(":id", user_id.to_string().as_str())])
        .unwrap()
}

pub async fn get_user_id(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    user_name: &str,
) -> Result<u32, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM users WHERE name = :name;") {
        Ok(s) => stmt = s,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    if !stmt.exists(&[(":name", user_name)]).unwrap() {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "User {} not found.",
            user_name
        )));
    }
    let iter = stmt.query_map(&[(":name", user_name)], |row| {
        Ok(SerdeUser {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    });
    match iter {
        Ok(mut ans) => Ok(ans.next().unwrap().expect("Unknown Error.").id),
        _ => Err(error_log::EXTERNAL::webmsg("Database Error.")),
    }
}

pub async fn get_user(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    user_id: u32,
) -> Result<SerdeUser, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM users WHERE id = :id;") {
        Ok(s) => stmt = s,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    if !stmt
        .exists(&[(":id", user_id.to_string().as_str())])
        .unwrap()
    {
        return Err(error_log::NOT_FOUND::webmsg(&format!(
            "User {} not found.",
            user_id
        )));
    }
    let iter = stmt.query_map(&[(":id", user_id.to_string().as_str())], |row| {
        Ok(SerdeUser {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    });
    match iter {
        Ok(mut ans) => Ok(ans.next().unwrap().expect("Unknown Error.")),
        _ => Err(error_log::EXTERNAL::webmsg("Database Error.")),
    }
}

pub async fn update_user(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    user_id: u32,
    user_name: &str,
) -> HttpResponse {
    println!("Users: Updating User...");
    let mut user: SerdeUser;
    match get_user(pool.clone(), user_id).await {
        Ok(ans) => {
            user = ans;
        }
        Err(e) => {
            return e;
        }
    };
    if !user_name.eq(&user.name) {
        //Update the names if they are not the same one.
        if user_name_exists(pool.clone(), user_name).await {
            return error_log::INVALID_ARGUMENT::webmsg(&format!(
                "User name '{}' already exists.",
                user_name
            ));
        } else {
            println!("updating now. {}: {} -> {}", user_id, user.name, user_name);
            let data = pool.lock().await.get().unwrap();
            let _ = data.execute(
                "UPDATE users SET name = ?1 WHERE id = ?2;",
                params![user_name.to_string(), user_id as i32],
            );
        }
    }
    user.name = user_name.to_string();
    HttpResponse::Ok().body(serde_json::to_string_pretty(&user).unwrap())
}

pub async fn create_user(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
    user_name: &str,
    ids: Data<Arc<Mutex<Ids>>>,
) -> Result<SerdeUser, HttpResponse> {
    println!("Users: Creating User...");

    let user_id: u32;
    // Check if user name exists before increase the counter.
    if user_name_exists(pool.clone(), user_name).await {
        return Err(error_log::INVALID_ARGUMENT::webmsg(&format!(
            "User name '{}' already exists.",
            user_name
        )));
    } else {
        user_id = ids.lock().await.usersid;
        ids.lock().await.usersid += 1;
        println!("User ID: {}", user_id);
        let data = pool.lock().await.get().unwrap();
        if let Err(_) = data.execute(
            "INSERT INTO users (id, name) VALUES (?1, ?2);",
            params![user_id, user_name],
        ) {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }

    Ok(SerdeUser {
        id: user_id,
        name: user_name.to_string(),
    })
}

pub async fn get_users(
    pool: Data<Mutex<Pool<SqliteConnectionManager>>>,
) -> Result<Vec<SerdeUser>, HttpResponse> {
    let data = pool.lock().await.get().unwrap();
    let mut stmt;
    match data.prepare("SELECT * FROM users ORDER BY id;") {
        Ok(s) => stmt = s,
        _ => {
            return Err(error_log::EXTERNAL::webmsg("Database Error."));
        }
    }
    let iter = stmt
        .query_map([], |row| {
            Ok(SerdeUser {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .expect("Unknown Error.");
    let mut users: Vec<SerdeUser> = vec![];
    for user in iter {
        users.push(user.unwrap());
    }
    Ok(users)
}
