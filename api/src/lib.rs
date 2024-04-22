#[macro_use]
extern crate rocket;
extern crate lazy_static;

#[macro_use]
mod marco;
mod db_pool;
mod routes;

use db_pool::Db;
use rocket::http::{Method, Status};
use rocket::Request;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use sea_orm_rocket::Database;

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
    ]);

    let cors = (rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    })
    .to_cors()
    .expect("could not generate CORS options");

    rocket::build()
        .attach(Db::init())
        .attach(cors)
        .mount("/", get_routes!(statistics))
        .mount("/", get_routes!(black_history))
        .mount("/", get_routes!(tools))
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
