#[macro_use]
extern crate rocket;
extern crate lazy_static;

#[macro_use]
mod marco;
mod db_pool;
mod meilisearch;
mod routes;
mod utils;
mod err;

use std::env;
use std::time::Duration;

use db_pool::Db;
use meilisearch::MeilisearchEngine;
use regex::RegexSet;
use rocket::fairing::AdHoc;
use rocket::http::{Header, Status};
use rocket::Request;
use rocket_cors::AllowedOrigins;
use sea_orm_rocket::Database;
use utils::cached_response::CacheTheResponse;

macro_rules! get_routes {
    ($x:ident) => {{
        routes::$x::routes()
    }};
}

#[catch(default)]
fn error_handler(status: Status, req: &Request<'_>) -> (Status, serde_json::Value) {
    let headers: Vec<_> = req.headers().iter().map(|x| (x.name, x.value)).collect();

    (
        status,
        serde_json::json!({
            "reason": status.reason(),
            "status": status,
            "request": {
                "origin": req.uri(),
                "headers": headers,
                "method": req.method(),
            }
        }),
    )
}

#[tokio::main]
async fn start() -> Result<(), rocket::Error> {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "https://aws-benben.imken.dev",
        "https://benben.imken.dev",
        "https://3000-immccn123-lgfeedfronten-3um3trd56q9.ws-us110.gitpod.io/",
        "https://next-benben.imken.dev/",
        "https://benben.sbs",
        "https://www.benben.sbs",
    ]);

    let cors = (rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    })
    .to_cors()
    .expect("could not generate CORS options");

    rocket::build()
        .attach(Db::init())
        .attach(MeilisearchEngine::init(
            env::var("MEILISEARCH_URL").expect("MEILISEARCH_URL is not set"),
            env::var("MEILISEARCH_KEY").ok(),
        ))
        .attach(CacheTheResponse {
            cache_time: Some(Duration::from_secs(60 * 60 * 24)),
            routes: RegexSet::new([r#"^/proxy/userPic/(.*)$"#])
                .expect("Cache the response: could not create the RegexSet"),
        })
        .attach(AdHoc::on_request("Save Request Time", |req, _| {
            Box::pin(async move {
                req.add_header(Header::new(
                    "X-Request-Time",
                    chrono::Utc::now().to_rfc3339(),
                ));
            })
        }))
        .attach(AdHoc::on_response("Add Execute Time", |req, resp| {
            Box::pin(async move {
                if let Some(time) = req.headers().get_one("X-Request-Time") {
                    if let Ok(request_time) = chrono::DateTime::parse_from_rfc3339(time) {
                        resp.adjoin_header(Header::new(
                            "X-Execute-Time",
                            chrono::Utc::now()
                                .signed_duration_since(request_time)
                                .num_milliseconds()
                                .to_string(),
                        ));
                    }
                }
            })
        }))
        .attach(cors)
        .mount("/", get_routes!(statistics))
        .mount("/", get_routes!(black_history))
        .mount("/", get_routes!(tools))
        .mount("/", get_routes!(proxy))
        .mount("/", get_routes!(process))
        .mount("/", rocket_cors::catch_all_options_routes())
        .register("/", catchers![error_handler])
        .launch()
        .await
        .map(|_| ())
}

pub fn main() {
    let result = start();

    println!("Rocket: deorbit.");

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
