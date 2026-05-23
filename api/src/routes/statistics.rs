use std::sync::LazyLock;

use crate::db_pool::Db;
use crate::err::AppError;
use crate::middleware::redis::{self, Commands};

use chrono::Duration;
use entity::feed::{self as benben};
use rocket_db_pools::Connection;
use sea_orm::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatisticsResponse {
    total_count: u64,
    today_count: u64,
}

pub static STATISTICS_LOCK: LazyLock<RwLock<()>> = LazyLock::new(|| RwLock::new(()));

#[get("/")]
async fn get_statistics(
    redis_conn: redis::Client<'_>,
    conn: Connection<Db>,
) -> Result<Json, AppError> {
    let mut redis_conn = redis_conn.into_inner().get_connection();

    {
        let _lock = STATISTICS_LOCK.read().await;

        if let Ok(redis_conn) = &mut redis_conn {
            let cached: Result<String, _> = redis_conn.get(format!("statistics"));
            if let Ok(cached) = cached {
                let json: Json = serde_json::from_str(&cached).expect("unexpected non-JSON string");
                return Ok(json);
            }
        }
    }

    let lock = STATISTICS_LOCK.write().await;

    if let Ok(redis_conn) = &mut redis_conn {
        let cached: Result<String, _> = redis_conn.get(format!("statistics"));
        if let Ok(cached) = cached {
            let json: Json = serde_json::from_str(&cached).expect("unexpected non-JSON string");
            return Ok(json);
        }
    }

    let db = &conn.into_inner();
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

    let result = json!(resp);

    if let Ok(redis_conn) = &mut redis_conn {
        redis_conn
            .set_ex::<String, String, String>(
                format!("statistics"),
                result.to_string(),
                Duration::hours(1).num_seconds() as u64,
            )
            .unwrap();
    }

    drop(lock);

    Ok(json!(result))
    // LOCK-ORDER: lock releases here
}

export_routes![get_statistics];
