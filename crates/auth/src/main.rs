use auth::AppState;
use auth::config::Config;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

fn load_templates(tera: &mut Tera) {
    for entry in std::fs::read_dir("templates").expect("Failed to read templates dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("html") {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            tera.add_template_file(path, Some(&name))
                .unwrap_or_else(|e| panic!("Failed to load template {name}: {e}"));
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Config::from_env();

    let mut tera = Tera::new();
    load_templates(&mut tera);

    let state = AppState {
        tera: Arc::new(tera),
        config: Arc::new(config.clone()),
    };

    let mut app = Router::new()
        .route("/flow/auth", get(auth::handlers::flow::auth))
        .route("/flow/callback", get(auth::handlers::flow::callback))
        .route("/api/refresh", post(auth::handlers::flow::refresh))
        .nest_service("/static", ServeDir::new("static"));

    #[cfg(debug_assertions)]
    {
        app = app
            .route("/tools/sign", get(auth::handlers::tools::sign_tool))
            .route("/tools/api/derive", post(auth::handlers::tools::api_derive))
            .route(
                "/tools/api/sign",
                post(auth::handlers::tools::api_sign_credential),
            );
    }

    let app = app.with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("Failed to bind address");

    println!("SSO Server running at {}", config.base_url);
    axum::serve(listener, app).await.expect("Server error");
}
