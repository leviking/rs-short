use std::collections::HashMap;
use std::fs::{OpenOptions, File};
use std::io::{self, BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::sync::Mutex;
use warp::Filter;

const DB_FILE: &str = "data.db";

lazy_static::lazy_static! {
    static ref DATA: Mutex<HashMap<String, String>> = {
        let mut data = HashMap::new();
        if let Ok(file) = File::open(DB_FILE) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    data.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        Mutex::new(data)
    };
}

fn main() {
    let set = warp::post()
        .and(warp::body::bytes())
        .map(|bytes: bytes::Bytes| {
            let data = bytes.to_vec();
            let key = gen_key();
            let mut map = DATA.lock().unwrap();
            map.insert(key.clone(), String::from_utf8(data).unwrap());
            warp::reply::html(key)
        });

    let get = warp::path::param()
        .map(|key: String| {
            let map = DATA.lock().unwrap();
            match map.get(&key) {
                Some(value) => warp::reply::html(value.clone()),
                None => warp::reply::with_status(
                    "Not Found",
                    warp::http::StatusCode::NOT_FOUND,
                ),
            }
        });

    let routes = set.or(get);
    warp::serve(routes).run(SocketAddr::from(([127, 0, 0, 1], 3000)));
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
