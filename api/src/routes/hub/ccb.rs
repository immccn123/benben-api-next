use std::time::{SystemTime, UNIX_EPOCH};

use entity::feed as benben;

use rand::SeedableRng;
use rand::{prelude::*, rngs::StdRng};
use rocket_db_pools::Connection;
use sea_orm::{prelude::*, Statement};
use serde_json::json;

use crate::{db_pool::Db, err::AppError};

#[get("/ccb/getRandom")]
async fn random3(conn: Connection<Db>) -> Result<Json, AppError> {
    let db = &conn.into_inner();

    let result = db
        .query_all(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            indoc::indoc! {r#"
                SELECT
                    feed."id"
                FROM
                    "feed"
                    LEFT JOIN feed_takedown ON feed."id" = feed_takedown."feed_id"
                WHERE
                    feed.grub_time - feed."time" < INTERVAL '24 hours'
                    AND feed_takedown."feed_id" IS NULL
                ORDER BY
                    feed."time" DESC
                LIMIT 10000 * 50
            "#},
            [],
        ))
        .await?
        .into_iter()
        .map(|x| {
            x.try_get_by_index::<i32>(0)
                .expect("the query is guranteed that here is unreachable")
        })
        .collect::<Vec<_>>();

    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let seed = since_the_epoch.as_secs() as u64;
    let mut rng = StdRng::seed_from_u64(seed);

    let result = result
        .sample(&mut rng, 10)
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();

    let benbens = benben::Entity::find()
        .filter(benben::Column::Id.is_in(result))
        .all(db)
        .await?;

    Ok(json!(benbens))
}

export_routes![random3];
