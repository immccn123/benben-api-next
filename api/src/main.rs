use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[macro_use]
extern crate rocket;
extern crate lazy_static;

use crate::middleware::{counter::CounterFairing, env_exposer::EnvExposer, redis::RedisFairing};
use shadow_rs::shadow;

// Declare Modules
#[macro_use]
mod marco;
mod db_pool;
mod err;
mod middleware;
mod routes;
mod utils;
shadow!(build);

#[allow(unused)]
use std::env;
use std::fs::File;
use std::io::Read;

use db_pool::Db;
use rocket::fairing::AdHoc;
use rocket::http::{Header, Status};
use rocket::Request;
use rocket_cors::AllowedOrigins;
use rocket_db_pools::Database;

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
            "request": { "origin": req.uri(), "headers": headers, "method": req.method() }
        }),
    )
}

#[tokio::main]
async fn start() -> Result<(), rocket::Error> {
    let mut cors_allow = vec![
        String::from("https://aws-benben.imken.dev"),
        String::from("https://benben.imken.dev"),
        String::from("https://next-benben.imken.dev/"),
        String::from("https://benben.sbs"),
        String::from("https://www.benben.sbs"),
        String::from("https://aws.benben.sbs"),
    ];

    if let Ok(file) = &mut File::open(".cors_allow") {
        let mut buf = String::new();
        if let Ok(_) = file.read_to_string(&mut buf) {
            cors_allow.extend(buf.lines().map(|x| {
                println!("CORS: {x}");
                x.to_string()
            }));
        }
    }

    let allowed_origins = AllowedOrigins::some_exact(cors_allow.as_ref());

    let cors = (rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    })
    .to_cors()
    .expect("could not generate CORS options");

    rocket::build()
        .attach(Db::init())
        .attach(RedisFairing::init(
            env::var("REDIS_URL").expect("REDIS url not set"),
        ))
        .attach(CounterFairing)
        .attach(EnvExposer::new(&["ANON_PRIVATE_KEY", "ANON_PUBLIC_KEY"]))
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
        .mount("/statistics", get_routes!(statistics))
        .mount("/blackHistory", get_routes!(black_history))
        .mount("/tools", get_routes!(tools))
        .mount("/search", get_routes!(search))
        .mount("/list", get_routes!(list))
        .mount("/hub", get_routes!(hub))
        .mount("/user", get_routes!(user))
        .mount("/diagnostic", get_routes!(diagnostic))
        .mount("/", rocket_cors::catch_all_options_routes())
        .register("/", catchers![error_handler])
        .launch()
        .await
        .map(|_| ())
}

fn main() {
    dotenvy::dotenv().ok();

    let result = start();

    println!("Rocket: deorbit.");

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
