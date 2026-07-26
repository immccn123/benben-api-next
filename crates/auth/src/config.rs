use std::env;

#[derive(Clone)]
pub struct Config {
    pub base_url: String,
    pub listen_addr: String,
    pub client_verify_key: String,
    pub jwt_sign_key: String,
    pub jwt_verify_key: String,
    pub cpoauth_base: String,
    pub cpoauth_client_id: String,
    pub cpoauth_client_secret: String,
    pub cpoauth_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            base_url: env::var("AUTH_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
            listen_addr: env::var("AUTH_LISTEN_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            client_verify_key: env::var("AUTH_CLIENT_VERIFY_KEY")
                .expect("AUTH_CLIENT_VERIFY_KEY must be set"),
            jwt_sign_key: env::var("AUTH_JWT_SIGN_KEY").expect("AUTH_JWT_SIGN_KEY must be set"),
            jwt_verify_key: env::var("AUTH_JWT_VERIFY_KEY")
                .expect("AUTH_JWT_VERIFY_KEY must be set"),
            cpoauth_base: env::var("CPOAUTH_BASE_URL")
                .unwrap_or_else(|_| "https://cpoauth.com".to_string()),
            cpoauth_client_id: env::var("CPOAUTH_CLIENT_ID")
                .expect("CPOAUTH_CLIENT_ID must be set"),
            cpoauth_client_secret: env::var("CPOAUTH_CLIENT_SECRET")
                .expect("CPOAUTH_CLIENT_SECRET must be set"),
            cpoauth_redirect_uri: env::var("CPOAUTH_REDIRECT_URI")
                .expect("CPOAUTH_REDIRECT_URI must be set"),
        }
    }
}
