use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use crate::db_pool::Db;
use crate::err::AppError;
use crate::middleware::redis::{self, Commands};

use chrono::Datelike;
use chrono::Duration;
use chrono::Utc;
use dashmap::DashMap;
use entity::takedown_batches;
use entity::{feed as benben, feed_takedown as benben_takedown};
use lazy_static::lazy_static;
use rand::RngExt;
use regex::Regex;
use rocket::http;
use rocket_db_pools::Connection;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::sea_query::Func;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::FromQueryResult;
use sea_orm::{EntityTrait, QueryFilter};
use serde_json::json;
use tokio::sync::Mutex;

lazy_static! {
    static ref MENTION_REGEXP: Regex =
        Regex::new(r"@\[(\S+)\]\(/user/(\d+)\)").expect("could not compile regex");
}

#[get("/at/<username>")]
async fn who_at_me(conn: Connection<Db>, username: &str) -> Result<Json, AppError> {
    let db = &conn.into_inner();
    let time_after = Utc::now() - Duration::days(1);

    let username: String = username.to_lowercase();

    let benbens = benben::Entity::find()
        .select()
        .left_join(benben_takedown::Entity)
        .filter(benben::Column::Time.gte(time_after))
        .filter(
            SimpleExpr::from(Func::lower(Expr::col(benben::Column::Content)))
                .like(format!("%{}%", &username)),
        )
        .filter(benben_takedown::Column::FeedId.is_null())
        .order_by_desc(benben::Column::Time)
        .all(db)
        .await?;

    let mut result = vec![];

    for benben in &benbens {
        let parts = MENTION_REGEXP.captures_iter(&benben.content);
        for caps in parts {
            let (_full, [name, _user_id]) = caps.extract();
            if name.to_lowercase() == username {
                result.push(benben);
                break;
            }
        }
    }

    Ok(json!(result))
}

#[get("/heatmap/<uid>")]
async fn get_heatmap(conn: Connection<Db>, uid: i32) -> Result<Json, AppError> {
    use chrono::NaiveDate;

    let db: &DatabaseConnection = &conn.into_inner();
    let today = Utc::now().naive_utc().date();
    let duration = 365 - (8 - today.weekday().num_days_from_sunday()) % 7;
    // let duration = 365;
    // let mut daily_counts = Vec::new();
    let past_date = (today - Duration::days(duration.into()))
        .and_hms_opt(0, 0, 0)
        .unwrap();

    #[derive(FromQueryResult, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct QueryResult {
        date: NaiveDate,
        count: i64,
    }

    impl std::cmp::PartialOrd for QueryResult {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.date.cmp(&other.date))
        }
    }
    impl std::cmp::Ord for QueryResult {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.date.cmp(&other.date)
        }
    }

    let mut result = benben::Entity::find()
        .filter(benben::Column::UserId.eq(uid))
        .filter(benben::Column::Time.gte(past_date))
        .filter(benben::Column::Time.lt(today.succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap()))
        .select_only()
        .column_as(Expr::cust("DATE(time)"), "date") // 仅提取日期部分
        .column_as(Expr::cust("COUNT(*)"), "count") // 计算每天的数量
        .group_by(Expr::cust("date")) // 按日期分组
        .order_by_asc(Expr::cust("date")) // 按日期升序排列
        .into_model::<QueryResult>()
        .all(db)
        .await?;

    result.sort();

    let mut date_counts = Vec::new();
    let mut current_date = past_date.date();
    let mut count_iter = result.iter();

    if let Some(mut current_count) = count_iter.next() {
        while current_date <= today {
            if current_count.date < current_date {
                // 日期不匹配，添加 count: 0
                date_counts.push(QueryResult {
                    date: current_date,
                    count: 0,
                });
                current_date = current_date.succ_opt().unwrap(); // 增加一天
            } else if current_count.date == current_date {
                // 日期匹配，添加当前计数
                date_counts.push(QueryResult {
                    date: current_count.date,
                    count: current_count.count,
                });
                current_date = current_date.succ_opt().unwrap(); // 增加一天
                current_count = if let Some(res) = count_iter.next() {
                    res
                } else {
                    break;
                };
            } else {
                date_counts.push(QueryResult {
                    date: current_date,
                    count: 0,
                });
                current_date = current_date.succ_opt().unwrap(); // 增加一天
            }
        }
    }

    Ok(json!(date_counts))
}

#[get("/getFeed/<feed_id>")]
async fn get_feed(conn: Connection<Db>, feed_id: i32) -> Result<(http::Status, Json), AppError> {
    let db = &conn.into_inner();

    let takedown = benben_takedown::Entity::find()
        .filter(benben_takedown::Column::FeedId.eq(feed_id))
        .find_also_related(takedown_batches::Entity)
        .one(db)
        .await?;

    let benben = benben::Entity::find_by_id(feed_id).one(db).await?;

    if let Some(takedown) = takedown {
        let info = takedown.1.expect("takedown batch not found");

        Ok((
            http::Status::Ok,
            json!({
                "takedown": true,
                "takedown_info": info,
            }),
        ))
    } else if let Some(benben) = benben {
        Ok((http::Status::Ok, json!(benben)))
    } else {
        Ok((
            http::Status::NotFound,
            json!({ "detail": "Benben not found" }),
        ))
    }
}

#[get("/getRandom")]
async fn get_random(conn: Connection<Db>) -> Result<(http::Status, Json), AppError> {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(FromQueryResult)]
    struct CountQueryResult {
        count: i32,
    }

    let db = &conn.into_inner();

    let total = benben::Entity::find()
        .select_only()
        .expr_as_(Func::max(Expr::col(benben::Column::Id)), "count")
        .into_model::<CountQueryResult>()
        .one(db)
        .await?
        .unwrap() // this will not happend
        .count;

    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let seed = since_the_epoch.as_secs() as u64;

    let mut rng = StdRng::seed_from_u64(seed);

    let id = loop {
        let id = rng.random_range(0..total);
        if let None = benben_takedown::Entity::find()
            .filter(benben_takedown::Column::FeedId.eq(id))
            .find_also_related(takedown_batches::Entity)
            .one(db)
            .await?
        {
            break id;
        }
    };

    let benben = benben::Entity::find_by_id(id).one(db).await?;

    if let Some(benben) = benben {
        Ok((http::Status::Ok, json!(benben)))
    } else {
        Ok((
            http::Status::NotFound,
            json!({"detail": "Benben not found"}),
        ))
    }
}

static UID_LOCKS: OnceLock<DashMap<i32, Arc<Mutex<()>>>> = OnceLock::new();

fn get_uid_locks() -> &'static DashMap<i32, Arc<Mutex<()>>> {
    UID_LOCKS.get_or_init(DashMap::new)
}

/// 权重：被提及、主动提及 = 100
/// 收集距离最新犇犇近 60 天 / 100 条（取最多）的数据，
/// 最新的 30 天 / 50 条权重为 100，其他的按条数差递减（向下取整），直到 25。
/// 总权重 >= 5 的才显示
#[get("/circle/<uid>")]
async fn get_activity_circle(
    redis_conn: redis::Client<'_>,
    dbconn: Connection<Db>,
    uid: i32,
) -> Result<Json, AppError> {
    let mut redis_conn = redis_conn.into_inner().get_connection();
    let cache_key = format!("circle/{uid}");

    if let Ok(conn) = &mut redis_conn {
        use redis::Commands;
        if let Ok(cached) = conn.get::<_, String>(&cache_key) {
            let mut json: Json = serde_json::from_str(&cached).expect("unexpected non-JSON string");
            json["cacheHit"] = json!(true);
            return Ok(json);
        }
    }

    let locks = get_uid_locks();
    let lock = locks
        .entry(uid)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();

    let _guard = lock.lock().await;

    if let Ok(conn) = &mut redis_conn {
        use redis::Commands;
        if let Ok(cached) = conn.get::<_, String>(&cache_key) {
            let mut json: Json = serde_json::from_str(&cached).expect("unexpected non-JSON string");
            json["cacheHit"] = json!(true);
            return Ok(json);
        }
    }

    let db = &dbconn.into_inner();
    let now = Utc::now();
    let past_60_days = now - Duration::days(60);
    let past_30_days = now - Duration::days(30);

    let mut pages = entity::circle_mention::Entity::find()
        .filter(
            Condition::any()
                .add(entity::circle_mention::Column::Sender.eq(uid as i32))
                .add(entity::circle_mention::Column::Mentioned.eq(uid as i32)),
        )
        .order_by_desc(entity::circle_mention::Column::Time)
        .paginate(db, 500);

    let mut weight_map: BTreeMap<i32, f64> = BTreeMap::new();
    let mut filter: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    let mut benben_flag: BTreeSet<i64> = BTreeSet::new();

    while let Some(data) = pages.fetch_and_next().await? {
        for mention in data {
            let entity::circle_mention::Model {
                sender,
                mentioned,
                feed_id,
                time,
            } = mention;

            let ok = filter.insert((sender, mentioned, feed_id));
            if !ok {
                continue; // 已经处理过的记录，跳过
            }

            let time_delta = (time - past_30_days.naive_local()).num_seconds() as f64;
            let total_time = Duration::days(30).num_seconds() as f64;
            let weight = f64::min(
                f64::max(3., 3. + (time_delta / total_time) * (100. - 3.)),
                100.0,
            );

            if benben_flag.len() < 100 || mention.time.ge(&past_60_days.naive_local()) {
                benben_flag.insert(mention.feed_id.into());

                if mentioned == uid && sender != uid {
                    // 被 at 且 非本人发送
                    *weight_map.entry(sender).or_insert(0.) += weight * 0.6;
                } else if mentioned != uid && sender == uid {
                    // 本人发送并且不是 at 自己
                    *weight_map.entry(mentioned).or_insert(0.) += weight;
                }
            } else {
                break;
            }
        }
    }

    let mut result: Vec<_> = weight_map.into_iter().collect::<Vec<_>>();
    result.sort_unstable_by(|(_, v1), (_, v2)| {
        v1.partial_cmp(v2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });

    let result_arr: Vec<_> = result
        .into_iter()
        .map(|(k, v)| {
            json!({
                "uid": k,
                "weight": v,
            })
        })
        .collect();

    let result_json = json!({
        "result": result_arr,
        "benbenCnt": benben_flag.len(),
        "cacheHit": false,
    });

    if let Ok(redis_conn) = &mut redis_conn {
        redis_conn
            .set_ex::<String, String, String>(
                format!("circle/{uid}"),
                result_json.to_string(),
                Duration::hours(8).num_seconds() as u64,
            )
            .unwrap();
    }
    Ok(result_json)
}
export_routes![
    who_at_me,
    get_heatmap,
    get_feed,
    get_random,
    get_activity_circle,
];
