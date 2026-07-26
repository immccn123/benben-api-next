pub mod config;
pub mod crypto;
pub mod handlers;

use config::Config;
use std::sync::Arc;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub config: Arc<Config>,
}
