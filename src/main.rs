use std::net::SocketAddr;
use warp::{Filter, http::StatusCode};
use sqlx::PgPool;
use serde::Deserialize;
use rand::Rng;

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
    .and(warp::filters::addr::remote())
    .and_then(handle_post);

    let get = warp::path::param()
        .and(warp::any().map(move || get_db_pool.clone()))
        .and_then(handle_get);

    // New route to serve the static index.html file
    let static_file = warp::path::end()
        .and(warp::fs::file("./static/index.html"));

    // Combine the routes
    let routes = post.or(get).or(static_file);

    warp::serve(routes).run(SocketAddr::from(([0, 0, 0, 0], 3000))).await;

    Ok(())
}

async fn handle_post(form: LinkForm, pool: PgPool, addr: Option<SocketAddr>) -> Result<impl warp::Reply, warp::Rejection> {
    let user = form.user.unwrap_or_else(|| {
        addr.map_or(String::new(), |a| a.ip().to_string())
    });
    
    match set_key(&form.url, Some(&user), &pool).await {
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
    let result = sqlx::query!("UPDATE urls SET visit_count = visit_count + 1 WHERE key = $1 RETURNING value", key)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|r| r.value))
}

fn gen_key() -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    
    let key: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect();

    key
}