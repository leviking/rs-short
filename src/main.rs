// Importing necessary libraries and modules
use warp::{self, Filter, http::StatusCode}; // Import Warp web framework
use sqlx::PgPool; // Import SQLx PostgreSQL pool
use serde::Deserialize; // Import Serde for deserialization
use rand::Rng; // Import the random number generator

// Constants
const KEY_LEN: usize = 4; // Length of generated keys
const MAX_RETRIES: usize = 10; // Maximum number of retries for generating unique keys
const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"; // Characters to use in generated keys

// Custom error type
#[derive(Debug)]
struct CustomError(String);

// Deserialization struct for handling incoming JSON data
#[derive(Debug, Deserialize)]
struct LinkForm {
    url: String,
    user: Option<String>,
}
// Implementing Reject trait for the custom error type
impl warp::reject::Reject for CustomError {}

// Entry point of the program
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from a .env file
    dotenv::dotenv().ok();

    // Retrieve the database URL from the environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a PostgreSQL database connection pool
    let db_pool = PgPool::connect(&database_url).await?;

    // Clone the database pool for use in different routes
    let post_db_pool = db_pool.clone();
    let get_db_pool = db_pool.clone();

    // Define the POST route for handling link submissions
    let post = warp::post()
        .and(warp::body::json())
        .and(warp::filters::addr::remote().map(|addr: Option<std::net::SocketAddr>| addr.map(|a| a.ip().to_string()).unwrap_or_default()))
        .and(warp::any().map(move || post_db_pool.clone()))
        .and_then(handle_post);

    // Define the GET route for redirecting to stored links
    let get = warp::path::param::<String>()
        .and(warp::any().map(move || get_db_pool.clone()))
        .and_then(handle_get);

    // Define a route for serving a static HTML file
    let static_file = warp::path::end()
        .and(warp::fs::file("./static/index.html"));

    // Combine all routes into a single router
    let routes = post.or(get).or(static_file);

    // Start the Warp web server on IP [0, 0, 0, 0] and port 3000
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;

    Ok(())
}

// Handler for POST requests to create short links
async fn handle_post(form: LinkForm, ip: String, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    // Determine the user based on the submitted data or IP address
    let user = form.user.unwrap_or(ip);

    // Attempt to set a key (short link) in the database
    match set_key(&form.url, Some(&user), &pool).await {
        Ok(key) => Ok(warp::reply::html(key)), // Return the generated key as an HTML response
        Err(_) => Err(warp::reject::custom(CustomError("Error saving key".to_string()))), // Return a custom error if key insertion fails
    }
}

// Handler for GET requests to redirect to the full URL
async fn handle_get(key: String, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    // Attempt to retrieve the full URL associated with the provided key
    match get_key(&key, &pool).await {
        Ok(Some(value)) => {
            // If a full URL is found, build a redirect response
            let reply = warp::http::Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", value)
                .body("")
                .unwrap();
            Ok(reply)
        }
        _ => Err(warp::reject::not_found()), // Return a 404 not found response if no URL is found
    }
}

// Function to generate a unique key for short links
async fn set_key(value: &str, user: Option<&str>, pool: &PgPool) -> Result<String, sqlx::Error> {
    let mut counter = 0; // Initialize a counter for retry attempts

    // Loop until a unique key is generated or the maximum number of retries is reached
    loop {
        let key = gen_key(counter); // Generate a key with an optional additional length

        // Attempt to insert the key-value pair into the database
        match sqlx::query!("INSERT INTO urls (key, value, user_id) VALUES ($1, $2, $3)", key, value, user)
            .execute(pool)
            .await
        {
            Ok(_) => return Ok(key), // Successfully inserted, return the key
            Err(_) if counter < MAX_RETRIES => {
                // Increment the counter and retry
                counter += 1;
            }
            Err(err) => return Err(err), // Return an error if retries exhausted
        }
    }
}

// Function to retrieve the full URL associated with a key
async fn get_key(key: &str, pool: &PgPool) -> Result<Option<String>, sqlx::Error> {
    // Attempt to update the visit count and retrieve the full URL from the database
    let result = sqlx::query!("UPDATE urls SET visit_count = visit_count + 1 WHERE key = $1 RETURNING value", key)
        .fetch_optional(pool)
        .await?;
    Ok(result.map(|r| r.value))
}

// Function to generate a random key with an optional additional length
fn gen_key(extra_len: usize) -> String {
    let total_len = KEY_LEN + extra_len; // Calculate the total length of the key
    let mut rng = rand::thread_rng();
    (0..total_len).map(|_| {
        let idx = rng.gen_range(0..CHARS.len());
        CHARS[idx] as char
    }).collect()
}