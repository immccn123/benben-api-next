use sea_orm;

use async_trait::async_trait;
use sea_orm::ConnectOptions;
use sea_orm_rocket::{rocket::figment::Figment, Database};
use std::time::Duration;

#[derive(Database, Debug)]
#[database("sea_orm")]
pub struct Db(SeaOrmPool);

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
    type Error = sea_orm::DbErr;

    type Connection = sea_orm::DatabaseConnection;

    async fn init(_figment: &Figment) -> Result<Self, Self::Error> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

        let mut options: ConnectOptions = ConnectOptions::new(url);
        options
            .max_connections(10)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(10));

        let conn = sea_orm::Database::connect(options).await?;

        Ok(SeaOrmPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}
