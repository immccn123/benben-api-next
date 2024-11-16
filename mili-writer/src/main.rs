extern crate dotenv;
use std::{io, time::Duration};

use dotenv::dotenv;

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
    idx.set_filterable_attributes(&[
        "id",
        "content",
        "username",
        "user_id",
        "time",
        "grab_time",
        ])
        .await?;
    println!("filterable indexes: {:?}", idx.get_filterable_attributes().await?);

    let mut progress = read_progress()?;

    loop {
        let filter = format!("id = {}", progress);

        let post_exists = idx
            .search()
            .with_filter(filter.as_str())
            .execute::<Benben>()
            .await;

        if let Ok(post_exists) = post_exists {
            // println!("{:?}", post_exists.hits);
            if post_exists.hits.len() != 0 {
                println!("Benben [id = {}] already saved", progress);
                progress += 1;

                if progress % 20 == 0 {
                    write_progress(progress)?;
                }
                continue;
            }
        }

        if let Some(benben) = entity::feed::Entity::find_by_id(progress).one(db).await? {
            use chrono::NaiveDateTime;
            use chrono_tz::Asia::Shanghai;

            #[inline]
            fn to_timestamp(date: NaiveDateTime) -> err::Result<i64> {
                Ok(date
                    .and_local_timezone(Shanghai)
                    .single()
                    .ok_or(())?
                    .timestamp())
            }

            idx.add_or_replace(
                &[Benben {
                    id: benben.id,
                    user_color: benben.user_color,
                    user_id: benben.user_id,
                    username: benben.username,
                    content: benben.content,
                    hash: benben.hash,
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
