use crate::db_pool::Db;
use crate::err::AppError;
use chrono::TimeZone;
use chrono::Utc;
use entity::{feed as benben, feed_takedown as benben_takedown};
use rocket::http;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityTrait, QueryFilter};
use sea_orm_rocket::Connection;
use serde_json::json;

#[get("/db?<keyword>&<senders>&<date_after>&<date_before>&<id_before>&<id_after>&<per_page>")]
async fn search_db(
    conn: Connection<'_, Db>,
    keyword: Option<&str>,
    senders: Option<Vec<i32>>,
    date_after: Option<i64>,
    date_before: Option<i64>,
    id_before: Option<i64>,
    id_after: Option<i64>,
    per_page: Option<u64>,
) -> Result<(http::Status, Json), AppError> {
    let db = conn.into_inner();
    let mut query = benben::Entity::find()
        .left_join(benben_takedown::Entity)
        .filter(benben_takedown::Column::FeedId.is_null())
        .select()
        .order_by_desc(benben::Column::Id);

    let per_page = match per_page {
        Some(x) => x,
        None => 50,
    };

    if let Some(keyword) = keyword {
        query = query.filter(benben::Column::Content.contains(keyword));
    }

    if let Some(senders) = senders {
        query = query.filter(benben::Column::UserId.is_in(senders));
    }

    if let Some(date_after) = date_after {
        let time = Utc.timestamp_micros(date_after).unwrap();
        query = query.filter(benben::Column::Time.gte(time));
    }

    if let Some(date_before) = date_before {
        let time = Utc.timestamp_micros(date_before).unwrap();
        query = query.filter(benben::Column::Time.lte(time));
    }

    if let Some(id_before) = id_before {
        query = query.filter(benben::Column::Id.gt(id_before));
    }

    if let Some(id_after) = id_after {
        query = query.filter(benben::Column::Id.lt(id_after));
    }

    query = query.limit(per_page);

    let result = query.all(db).await?;

    Ok((http::Status::Ok, json!(result)))
}

export_routes![search_db];
