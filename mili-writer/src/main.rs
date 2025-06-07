use chrono::NaiveDateTime;
use chrono_tz::Asia::Shanghai;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use std::{io, time::Duration};

use dotenvy::dotenv;

mod err;
use entity::search::Benben;
use err::Error;
use sea_orm::EntityTrait;

fn write_progress(progress: i32) -> io::Result<()> {
    use std::fs;

    fs::write("./progress.txt", progress.to_string())
}

fn read_progress() -> err::Result<i32> {
    use std::fs;

    let progress = match fs::read_to_string("./progress.txt") {
        Ok(p) => p,
        Err(_) => {
            write_progress(0)?;
            "1".to_string()
        }
    }
    .trim()
    .parse::<i32>()?;

    Ok(progress)
}

#[inline]
fn to_timestamp(date: NaiveDateTime) -> err::Result<i64> {
    Ok(date
        .and_local_timezone(Shanghai)
        .single()
        .ok_or(())?
        .timestamp())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    use sea_orm::{Database, DatabaseConnection};

    dotenv().ok();

    let db: &DatabaseConnection =
        &Database::connect(std::env::var("DATABASE_URL").expect("DATABASE URL is not set")).await?;

    db.ping().await?;

    let meili = meilisearch_sdk::Client::new(
        std::env::var("MEILISEARCH_URL").expect("MEILISEARCH_URL is not set"),
        std::env::var("MEILISEARCH_KEY").ok(),
    );

    meili.create_index("benben", Some("id")).await.ok();
    let idx = meili.index("benben");
    idx.set_filterable_attributes(&["id", "content", "username", "user_id", "time", "grab_time"])
        .await?;

    println!(
        "filterable indexes: {:?}",
        idx.get_filterable_attributes().await?
    );

    let mut progress = read_progress()?;

    loop {
        let check_filter = format!("id = {}", progress + 2500);

        let batch_check = idx
            .search()
            .with_filter(check_filter.as_str())
            .execute::<Benben>()
            .await;

        if let Ok(batch_check) = batch_check {
            if batch_check.hits.is_empty() {
                let batch = entity::feed::Entity::find()
                    .filter(entity::feed::Column::Id.gte(progress))
                    .filter(entity::feed::Column::Id.lt(progress + 2500))
                    .all(db)
                    .await?;

                let batch_data: Vec<Benben> = batch
                    .into_iter()
                    .map(|benben| Benben {
                        id: benben.id,
                        user_id: benben.user_id,
                        username: benben.username,
                        content: benben.content,
                        time: to_timestamp(benben.time).unwrap_or_default(),
                        grab_time: to_timestamp(benben.grab_time).unwrap_or_default(),
                    })
                    .collect();

                idx.add_or_replace(&batch_data, Some("id")).await?;

                println!("Batch indexed [id = {} to {}]", progress, progress + 2499);

                progress += 2500;

                write_progress(progress)?;
                continue;
            }
        }

        let filter = format!("id = {}", progress);

        let post_exists = idx
            .search()
            .with_filter(filter.as_str())
            .execute::<Benben>()
            .await;

        if let Ok(post_exists) = post_exists {
            if !post_exists.hits.is_empty() {
                println!("Benben [id = {}] already saved", progress);
                progress += 1;

                if progress % 20 == 0 {
                    write_progress(progress)?;
                }
                continue;
            }
        }

        if let Some(benben) = entity::feed::Entity::find_by_id(progress).one(db).await? {
            idx.add_or_replace(
                &[Benben {
                    id: benben.id,
                    user_id: benben.user_id,
                    username: benben.username,
                    content: benben.content,
                    time: to_timestamp(benben.time)?,
                    grab_time: to_timestamp(benben.grab_time)?,
                }],
                Some("id"),
            )
            .await?;

            println!("Saved [id = {}]", progress);

            progress += 1;

            if progress % 20 == 0 {
                write_progress(progress)?;
            }
        } else {
            println!("Benben [id = {}] not found, wait for 1 second", progress);
            tokio::time::interval(Duration::from_secs(5)).tick().await;
        }
    }
}
