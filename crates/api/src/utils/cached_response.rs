use std::time::Duration;

use rocket::fairing::{Fairing, Info, Kind};

#[allow(unused)]
pub struct CacheTheResponse {
    pub cache_time: Option<Duration>,
    pub routes: regex::RegexSet,
}

#[rocket::async_trait]
impl Fairing for CacheTheResponse {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Cache the response",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(
        &self,
        request: &'r rocket::Request<'_>,
        response: &mut rocket::Response<'r>,
    ) {
        if !self
            .routes
            .is_match(request.uri().path().to_string().as_str())
        {
            return;
        }

        if let Some(cache_time) = self.cache_time {
            response.set_raw_header(
                "Cache-Control",
                format!("public, max-age={}", cache_time.as_secs()),
            );
        }
    }
}
