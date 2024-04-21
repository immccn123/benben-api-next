use crate::db_pool::Db;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use entity::feed::{self as benben};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::http;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityTrait, QueryFilter};
use sea_orm_rocket::Connection;
use serde_json::json;

lazy_static! {
    static ref MENTION_REGEXP: Regex =
        Regex::new(r"@\[(\S+)\]\(/user/(\d+)\)").expect("could not compile regex");
}

#[get("/tools/at/<username>")]
async fn who_at_me(conn: Connection<'_, Db>, username: &str) -> JsonValue {
    let db = conn.into_inner();
    let time_after = Utc::now() - Duration::days(1);

    let benbens = benben::Entity::find()
        .select()
        .filter(benben::Column::Time.gte(time_after))
        .filter(benben::Column::Content.contains(username))
        .order_by_desc(benben::Column::Time)
        .all(db)
        .await
        .expect("error occured during querying who at me query");

    let mut result = vec![];
    let username = username.to_lowercase();

    for benben in &benbens {
        let parts = MENTION_REGEXP.captures_iter(&benben.content);
        for caps in parts {
            let (_full, [name, _user_id]) = caps.extract();
            if name.to_lowercase() == username {
                result.push(benben);
            }
        }
    }

    json!(result)
}

#[get("/tools/heatmap/<_uid>")]
async fn get_heatmap(_conn: Connection<'_, Db>, _uid: i32) {
    todo!("not impl yet");
}

#[get("/tools/getFeed/<feed_id>")]
async fn get_feed(conn: Connection<'_, Db>, feed_id: i32) -> (http::Status, JsonValue) {
    let db = conn.into_inner();

    let benben = benben::Entity::find_by_id(feed_id)
        .one(db)
        .await
        .expect("could not get benben by id");

    if let Some(benben) = Some(benben) {
        (http::Status::Ok, json!(benben))
    } else {
        (
            http::Status::NotFound,
            json!({"detail": "Benben not found"}),
        )
    }
}

#[get(
    "/search/db?<keyword>&<senders>&<date_after>&<date_before>&<id_before>&<id_after>&<per_page>"
)]
async fn search(
    conn: Connection<'_, Db>,
    keyword: Option<String>,
    senders: Option<Vec<i32>>,
    date_after: Option<i64>,
    date_before: Option<i64>,
    id_before: Option<i64>,
    id_after: Option<i64>,
    per_page: Option<u64>,
) -> (http::Status, JsonValue) {
    let db = conn.into_inner();
    let mut query = benben::Entity::find().select();
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

    let result = query.all(db).await.expect("could not search");

    (http::Status::Ok, json!(result))
}

export_routes![who_at_me, get_heatmap, get_feed, search];
