use std::{collections::BTreeSet, env};

use dotenvy::dotenv;
use entity::{circle_mention, feed};
use regex::Regex;
use sea_orm::{ActiveValue::Set, Database, EntityTrait, QueryOrder, prelude::*};

mod err;

#[tokio::main]
async fn main() -> Result<(), err::Error> {
    dotenv().ok();

    let db_conn = Database::connect(
        env::var("DATABASE_URL").expect("env variable `DATABASE_URL` should be set"),
    )
    .await?;

    let mention = Regex::new(r"@\[(\S+)\]\(/user/(\d+)\)").unwrap();

    loop {
        let begin = circle_mention::Entity::find()
            .order_by_desc(circle_mention::Column::FeedId)
            .one(&db_conn)
            .await?
            .map(|x| x.feed_id)
            .unwrap_or(0)
            + 1;

        // [begin, end), per_page = 100
        let mut end = begin + 500;

        loop {
            let begin = end - 500;
            let feeds = feed::Entity::find()
                .filter(feed::Column::Id.gte(begin))
                .filter(feed::Column::Id.lt(end))
                .order_by_asc(feed::Column::Id)
                .all(&db_conn)
                .await?;

            println!("saving: {begin}..{end}");

            for feed in &feeds {
                let mut mentions = BTreeSet::new();
                for cap in mention.captures_iter(&feed.content) {
                    let (_, [_, mentioned_uid]) = cap.extract();
                    let mentioned_uid = if let Ok(uid) = mentioned_uid.parse::<i32>() {
                        uid
                    } else {
                        continue;
                    };
                    mentions.insert(mentioned_uid);
                }
                for mention in mentions.iter() {
                    circle_mention::Entity::insert(circle_mention::ActiveModel {
                        feed_id: Set(feed.id),
                        sender: Set(feed.user_id),
                        mentioned: Set(*mention),
                        time: Set(feed.time),
                    })
                    .exec(&db_conn)
                    .await?;
                }
            }

            end += 500;

            if feeds.is_empty() {
                break;
            }
        }

        // sleep 3 mins
        tokio::time::sleep(std::time::Duration::from_secs(180)).await;
    }
}
