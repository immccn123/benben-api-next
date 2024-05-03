use entity::feed::{self as benben};
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryFilter, SelectColumns};
use serde::{Deserialize, Serialize};

use serde_json::json;

use sea_orm_rocket::Connection;

use crate::db_pool::Db;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatisticsResponse {
    total_count: u64,
    today_count: u64,
    total_user: u64,
    today_user: u64,
}


#[get("/statistics")]
async fn get_statistics(conn: Connection<'_, Db>) -> serde_json::Value {
    let db = conn.into_inner();
    let yesterday = chrono::Utc::now() - std::time::Duration::from_secs(24 * 60 * 60);

    let total_count = benben::Entity::find()
        .count(db)
        .await
        .expect("could not get count of the data");

    let today_count = benben::Entity::find()
        .filter(benben::Column::Time.gte(yesterday))
        .count(db)
        .await
        .expect("could not get count of today");

    let total_user = benben::Entity::find()
        .select_column(benben::Column::UserId)
        .distinct()
        .count(db)
        .await
        .expect("could not get total_user");

    let today_user = benben::Entity::find()
        .select_column(benben::Column::UserId)
        .filter(benben::Column::Time.gte(yesterday))
        .distinct()
        .count(db)
        .await
        .expect("could not get today_user");

    let resp = StatisticsResponse {
        today_count,
        today_user,
        total_count,
        total_user,
    };

    json!(resp)
}

#[get("/statistics/24h")]
async fn get_statistics_24h(_conn: Connection<'_, Db>) -> JsonValue {
    json!([])
}

#[get("/statistics/60d")]
async fn get_statistics_60d(_conn: Connection<'_, Db>) -> JsonValue {
    json!([])
}

export_routes![get_statistics, get_statistics_24h, get_statistics_60d];
