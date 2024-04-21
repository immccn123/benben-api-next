use std::collections::BTreeMap;

use entity::feed::{self as benben};
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{EntityOrSelect, QueryFilter};
use sea_orm_rocket::Connection;

use crate::db_pool::Db;
use serde_json::json;

#[get("/blackHistory/feed/<uid>?<per_page>&<page>")]
async fn get_history_feed(
    conn: Connection<'_, Db>,
    uid: u64,
    per_page: u64,
    page: u64,
) -> JsonValue {
    let db = conn.into_inner();
    let benbens = benben::Entity
        .select()
        .filter(benben::Column::UserId.eq(uid))
        .offset((page - 1) * per_page)
        .limit(per_page)
        .order_by_desc(benben::Column::Time)
        .all(db)
        .await
        .expect("could not get benbens");

    let count = benben::Entity
        .select()
        .filter(benben::Column::UserId.eq(uid))
        .count(db)
        .await
        .expect("could not get count of benbens by uid");

    json!({ "feeds": benbens, "count": count })
}

#[get("/blackHistory/usernames/<uid>")]
async fn get_history_username_by_uid(conn: Connection<'_, Db>, uid: u64) -> JsonValue {
    let db = conn.into_inner();
    let mut flag: BTreeMap<String, bool> = BTreeMap::new();

    let usernames: Vec<_> = benben::Entity::find()
        .select_column(benben::Column::Username)
        .filter(benben::Column::UserId.eq(uid))
        .all(db)
        .await
        .expect("could not get history usernames")
        .iter()
        .map(|x| x.username.clone())
        .collect();

    for name in usernames {
        flag.insert(name, true);
    }

    let result: Vec<_> = flag.keys().collect();

    json!(result)
}

#[get("/blackHistory/uids/<username>")]
async fn get_history_uid_by_username(conn: Connection<'_, Db>, username: String) -> JsonValue {
    let db = conn.into_inner();
    let mut flag: BTreeMap<i32, bool> = BTreeMap::new();

    let uids: Vec<_> = benben::Entity::find()
        .select_column(benben::Column::Username)
        .filter(benben::Column::Username.eq(username))
        .all(db)
        .await
        .expect("could not get history uids by username")
        .iter()
        .map(|x| x.user_id.clone())
        .collect();

    for uid in uids {
        flag.insert(uid, true);
    }

    let result: Vec<_> = flag.keys().collect();

    json!(result)
}

export_routes![
    get_history_feed,
    get_history_username_by_uid,
    get_history_uid_by_username
];
