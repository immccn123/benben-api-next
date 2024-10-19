extern crate dotenv;

use dotenv::dotenv;

use dialoguer::{theme::ColorfulTheme, Input};
use entity::{feed_takedown, takedown_batches};

use sea_orm::{Database, DatabaseConnection, DbErr, TransactionError};

async fn batch_takedown(
    db: &DatabaseConnection,
    feed_ids: Vec<i32>,
    takedown_reason: String,
    takedown_user_id: i32,
) -> Result<(), TransactionError<DbErr>> {
    use sea_orm::{DbErr, EntityTrait, Set, TransactionTrait};

    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            let new_batch = takedown_batches::ActiveModel {
                reason: Set(takedown_reason),
                takedown_user_id: Set(takedown_user_id),
                ..Default::default()
            };
            let batch_result = takedown_batches::Entity::insert(new_batch)
                .exec(txn)
                .await?;
            let batch_id = batch_result.last_insert_id;

            println!("已插入 batch = {batch_id}");

            for feed_id in feed_ids {
                let new_feed_takedown = feed_takedown::ActiveModel {
                    feed_id: Set(feed_id),
                    batch_id: Set(batch_id),
                    ..Default::default()
                };
                feed_takedown::Entity::insert(new_feed_takedown)
                    .exec(txn)
                    .await?;
            }

            Ok(())
        })
    })
    .await
}

#[tokio::main]
async fn start() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db =
        Database::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL is not set")).await?;

    // let feed_ids_to_takedown = vec![101, 102, 103];
    // let takedown_reason = String::from("违反社区规范");
    // let takedown_user_id = 1;

    let takedown_reason: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入下架原因")
        .interact_text()?;

    // 获取下架用户 ID
    let takedown_user_id: i32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入执行下架操作的用户 ID")
        .interact_text()?;

    // 选择需要下架的 feed 条目
    let selected_ids: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("输入需要下架的 feed 条目，`,` 分隔")
        .interact()?;

    let feed_ids_to_takedown: Vec<i32> = selected_ids
        .split(',')
        .filter_map(|id| id.trim().parse::<i32>().ok())
        .collect();

    batch_takedown(&db, feed_ids_to_takedown, takedown_reason, takedown_user_id).await?;
    println!("批量下架操作完成。");

    Ok(())
}

fn main() {
    if let Err(err) = start() {
        eprintln!("Error: {}", err);
    }
}
