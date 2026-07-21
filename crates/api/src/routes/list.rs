use crate::db_pool::Db;
use crate::err::{self, AppError};
use entity::{feed as benben, feed_takedown as benben_takedown};
use rocket::http;
use rocket_db_pools::Connection;
use sea_orm::{prelude::*, query::*, EntityTrait, QueryFilter};
use serde_json::json;

#[get("/?<uids>")]
async fn list(conn: Connection<Db>, uids: Option<&str>) -> Result<(http::Status, Json), AppError> {
    let db = &conn.into_inner();

    let mut query = benben::Entity::find()
        .left_join(benben_takedown::Entity)
        .filter(benben_takedown::Column::FeedId.is_null())
        .select()
        .order_by_desc(benben::Column::Time);

    match uids {
        Some(uid_str) => {
            let iter = uid_str.split(",").filter(|x| x.len() > 0);
            let mut uids = Vec::new();
            for item in iter {
                let uid = item.trim().parse::<i32>().map_err(|_| {
                    err::AppError::ParamValidateFailed("param contains non-integer.")
                })?;
                uids.push(uid);
            }

            query = query.filter(benben::Column::UserId.is_in(uids));
            query = query.limit(100);
        }
        None => {
            query = query.limit(100);
        }
    }

    let result = query.all(db).await?;

    Ok((http::Status::Ok, json!(result)))
}

export_routes![list];
