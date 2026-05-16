use ::redis::Commands;
use sea_orm_rocket::Connection;
use serde_json::json;

use crate::{build, db_pool::Db, err::AppError, middleware::redis};

#[get("/")]
async fn diagnostic(
    redis_conn: redis::Client<'_>,
    db: Connection<'_, Db>,
) -> Result<serde_json::Value, AppError> {
    let db = db.into_inner();
    let redis_server = redis_conn.into_inner();

    let database_status = if db.ping().await.is_ok() {
        "up"
    } else {
        "down"
    };

    let redis_status = if redis_server
        .get_connection()
        .and_then(|mut conn| conn.ping::<()>())
        .is_ok()
    {
        "up"
    } else {
        "down"
    };

    let commit_hash = build::COMMIT_HASH[0..7].to_owned();

    Ok(json!({
        "version": commit_hash,
        "database": database_status,
        "redis": redis_status,
        "$monitor": format!("db+{database_status}:redis+{redis_status}")
    }))
}

export_routes![diagnostic];
