pub use redis::Commands;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::FromRequest,
};

pub struct Client<'a>(&'a redis::Client);

pub struct RedisFairing {
    url: String,
}

#[rocket::async_trait]
impl Fairing for RedisFairing {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Redis Engine",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
        let conn =
            redis::Client::open(self.url.as_str()).expect("could not connect to redis server");

        Ok(rocket.manage(conn))
    }
}

impl RedisFairing {
    #[allow(unused)]
    pub fn init(url: impl Into<String>) -> Self {
        RedisFairing { url: url.into() }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Client<'r> {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        match req.rocket().state::<redis::Client>() {
            Some(conn) => rocket::request::Outcome::Success(Client(conn)),
            None => rocket::request::Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

impl<'a> Client<'a> {
    pub fn into_inner(self) -> &'a redis::Client {
        self.0
    }
}
