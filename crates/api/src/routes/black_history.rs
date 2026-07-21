use std::collections::BTreeMap;

use entity::{feed as benben, feed_takedown as benben_takedown};
use rocket_db_pools::Connection;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityOrSelect, QueryFilter};
use serde_json::json;

use crate::db_pool::Db;
use crate::err::AppError;

#[get("/feed/<uid>?<per_page>&<page>")]
async fn get_history_feed(
    db: Connection<Db>,
    uid: u64,
    per_page: u64,
    page: u64,
) -> Result<Json, AppError> {
    let db = &db.into_inner();

    let benbens = benben::Entity
        .select()
        .left_join(benben_takedown::Entity)
        .filter(benben_takedown::Column::FeedId.is_null())
        .filter(benben::Column::UserId.eq(uid))
        .offset((page - 1) * per_page)
        .limit(per_page)
        .order_by_desc(benben::Column::Time)
        .all(db)
        .await?;

    let count = benben::Entity
        .select()
        .filter(benben::Column::UserId.eq(uid))
        .count(db)
        .await?;

    Ok(json!({ "feeds": benbens, "count": count }))
}

#[get("/usernames/<uid>")]
async fn get_history_username_by_uid(db: Connection<Db>, uid: u64) -> Result<Json, AppError> {
    let mut flag: BTreeMap<String, bool> = BTreeMap::new();
    let db = &db.into_inner();

    let usernames: Vec<_> = benben::Entity::find()
        .select_column(benben::Column::Username)
        .filter(benben::Column::UserId.eq(uid))
        .all(db)
        .await?
        .iter()
        .map(|x| x.username.clone())
        .collect();

    for name in usernames {
        flag.insert(name, true);
    }

    let result: Vec<_> = flag.keys().collect();

    Ok(json!(result))
}

#[get("/uids/<username>")]
async fn get_history_uid_by_username(db: Connection<Db>, username: &str) -> Result<Json, AppError> {
    let db = &db.into_inner();
    let mut flag: BTreeMap<i32, bool> = BTreeMap::new();

    let uids: Vec<_> = benben::Entity::find()
        .select_column(benben::Column::Username)
        .filter(benben::Column::Username.eq(username))
        .all(db)
        .await?
        .iter()
        .map(|x| x.user_id.clone())
        .collect();

    for uid in uids {
        flag.insert(uid, true);
    }

    let result: Vec<_> = flag.keys().collect();

    Ok(json!(result))
}

export_routes![
    get_history_feed,
    get_history_username_by_uid,
    get_history_uid_by_username
];
