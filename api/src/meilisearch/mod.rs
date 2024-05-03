use std::borrow::Borrow;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::FromRequest,
};

pub struct Client<'a>(&'a meilisearch_sdk::Client);

pub struct MeilisearchEngine {
    host: String,
    api_key: Option<String>,
}

#[rocket::async_trait]
impl Fairing for MeilisearchEngine {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Meilisearch Engine",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
        let conn = meilisearch_sdk::Client::new(self.host.clone(), self.api_key.clone());

        if let Err(err) = conn.get_version().await {
            error!("failed to connect to the meilisearch server: {}", err);
            Err(rocket)
        } else {
            Ok(rocket.manage(conn))
        }
    }
}

impl MeilisearchEngine {
    pub fn init(host: impl Into<String>, api_key: Option<impl Into<String>>) -> Self {
        MeilisearchEngine {
            host: host.into(),
            api_key: match api_key {
                Some(x) => Some(x.into()),
                None => None,
            },
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Client<'r> {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        match req.rocket().state::<meilisearch_sdk::Client>() {
            Some(conn) => rocket::request::Outcome::Success(Client(conn)),
            None => rocket::request::Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

impl<'a> Client<'a> {
    pub fn into_inner(self) -> &'a meilisearch_sdk::Client {
        self.0
    }
}
