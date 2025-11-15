use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::FromRequest,
};
use std::sync::atomic::AtomicU64;

#[derive(Debug)]
pub struct HitCount {
    count: AtomicU64,
}

pub struct HitCountGuard<'r>(&'r HitCount);

pub struct CounterFairing;

#[rocket::async_trait]
impl Fairing for CounterFairing {
    fn info(&self) -> Info {
        Info {
            name: "Counter",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
        let cnt = HitCount {
            count: AtomicU64::new(0),
        };
        Ok(rocket.manage(cnt))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HitCountGuard<'r> {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        match req.rocket().state::<HitCount>() {
            Some(conn) => rocket::request::Outcome::Success(HitCountGuard(conn)),
            None => rocket::request::Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

impl<'a> HitCountGuard<'a> {
    pub fn increment(&self) -> u64 {
        self.0
            .count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1
    }
}
