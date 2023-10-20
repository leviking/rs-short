use std::net::SocketAddr;
use warp::{Filter, http::StatusCode};
use sqlx::PgPool;
use serde::Deserialize;

#[derive(Debug)]
struct CustomError(String);

#[derive(Debug, Deserialize)]
struct LinkForm {
    url: String,
    user: Option<String>,
}
impl warp::reject::Reject for CustomError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");    
    let db_pool = PgPool::connect(&database_url).await?;

    let post_db_pool = db_pool.clone();
    let get_db_pool = db_pool.clone();

    let post = warp::post()
        .and(warp::body::json())
        .and(warp::any().map(move || post_db_pool.clone()))
        .and_then(handle_post);


    let get = warp::path::param()
        .and(warp::any().map(move || get_db_pool.clone()))
        .and_then(handle_get);

    let routes = post.or(get);

    warp::serve(routes).run(SocketAddr::from(([127, 0, 0, 1], 3000))).await;

    Ok(())
}

async fn handle_post(form: LinkForm, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    match set_key(&form.url, form.user.as_deref(), &pool).await {
        Ok(key) => Ok(warp::reply::html(key)),
        Err(_) => Err(warp::reject::custom(CustomError("Error saving key".to_string()))),
    }
}

async fn handle_get(key: String, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    match get_key(&key, &pool).await {
        Ok(Some(value)) => {
            let reply = warp::http::Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", value)
                .body("")
                .unwrap();
            Ok(reply)
        }
        _ => Err(warp::reject::not_found()),
    }
}

async fn set_key(value: &str, user: Option<&str>, pool: &PgPool) -> Result<String, sqlx::Error> {
    let key = gen_key();
    sqlx::query!("INSERT INTO urls (key, value, user_id) VALUES ($1, $2, $3)", key, value, user)
        .execute(pool)
        .await?;
    Ok(key)
}

async fn get_key(key: &str, pool: &PgPool) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!("SELECT value FROM urls WHERE key = $1", key)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|r| r.value))
}

fn gen_key() -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut key = String::new();
    for _ in 0..4 {
        let idx = rand::random::<usize>() % chars.len();
        key.push(chars[idx]);
    }
    key
}
