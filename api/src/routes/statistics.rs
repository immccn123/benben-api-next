use entity::feed::{self as benben};
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryFilter};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db_pool::Db;
use crate::err::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatisticsResponse {
    total_count: u64,
    today_count: u64,
}

#[get("/statistics")]
async fn get_statistics(conn: Connection<'_, Db>) -> Result<JsonValue, AppError> {
    let db = conn.into_inner();
    let yesterday = chrono::Utc::now() - std::time::Duration::from_secs(24 * 60 * 60);

    let total_count = benben::Entity::find().count(db).await?;

    let today_count = benben::Entity::find()
        .filter(benben::Column::Time.gte(yesterday))
        .count(db)
        .await?;

    let resp = StatisticsResponse {
        today_count,
        total_count,
    };

    Ok(json!(resp))
}

export_routes![get_statistics];
