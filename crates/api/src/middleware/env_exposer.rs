use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::{FromRequest, Outcome},
};
use std::{collections::HashMap, env};

#[allow(unused)]
pub struct EnvStore(HashMap<String, String>);

#[allow(unused)]
pub struct EnvGuard<'r>(&'r EnvStore);

pub struct EnvExposer {
    keys_to_capture: Vec<String>,
}

impl EnvExposer {
    pub fn new(keys: &[&'static str]) -> Self {
        EnvExposer {
            keys_to_capture: keys.into_iter().map(|x| x.to_string()).collect(),
        }
    }
}

#[rocket::async_trait]
impl Fairing for EnvExposer {
    fn info(&self) -> Info {
        Info {
            name: "Configurable Environment Exposer",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
        let mut env_store = HashMap::<String, String>::new();

        for key in &self.keys_to_capture {
            match env::var(key) {
                Ok(value) => {
                    env_store.insert(key.clone(), value);
                }
                Err(e) => {
                    eprintln!(
                        "FATAL: Required environment variable '{}' not found or invalid: {}",
                        key, e
                    );
                    return Err(rocket);
                }
            }
        }

        Ok(rocket.manage(EnvStore(env_store)))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EnvGuard<'r> {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<EnvStore>() {
            Some(store) => Outcome::Success(EnvGuard(store)),
            None => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

impl<'a> EnvGuard<'a> {
    #[allow(unused)]
    pub fn get(&self, key: &str) -> Option<&str> {
        (self.0).0.get(key).map(|s| s.as_str())
    }
}
