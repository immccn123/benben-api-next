use axum::{extract::State, response::{Html, IntoResponse, Json, Response}};
use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::signature::Signer;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::crypto::derive;

pub async fn sign_tool(State(state): State<AppState>) -> Html<String> {
    let ctx = tera::Context::new();
    let html = state
        .tera
        .render("tools-sign.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Error</h1><pre>{e}</pre>"));
    Html(html)
}

#[derive(Deserialize)]
pub struct DeriveRequest {
    pub root_key: String,
    pub app_id: String,
}

pub async fn api_derive(Json(body): Json<DeriveRequest>) -> Response {
    let root_priv = match derive::root_priv_from_pem(&body.root_key) {
        Some(k) => k,
        None => return Json(json!({"error": "无法解析根私钥 PEM"})).into_response(),
    };

    let signing_key = derive::derive_signing_key(&root_priv, &body.app_id);
    let verifying_key = signing_key.verifying_key();

    let app_pub_pem = {
        let pub_key: p256::PublicKey = verifying_key.clone().into();
        pkcs8::EncodePublicKey::to_public_key_pem(&pub_key, pkcs8::LineEnding::LF)
            .unwrap_or_default()
    };

    let app_priv_pem = {
        pkcs8::EncodePrivateKey::to_pkcs8_pem(&signing_key, pkcs8::LineEnding::LF)
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    Json(json!({
        "app_pub_pem": app_pub_pem,
        "app_priv_pem": app_priv_pem,
    }))
    .into_response()
}

#[derive(Deserialize)]
pub struct SignCredentialRequest {
    pub root_key: String,
    pub app_id: String,
    pub callback_url: String,
}

pub async fn api_sign_credential(Json(body): Json<SignCredentialRequest>) -> Response {
    let root_priv = match derive::root_priv_from_pem(&body.root_key) {
        Some(k) => k,
        None => return Json(json!({"error": "无法解析根私钥 PEM"})).into_response(),
    };

    let signing_key = derive::derive_signing_key(&root_priv, &body.app_id);
    let message = format!("ba+{}+{}", body.app_id, body.callback_url);
    let sig: p256::ecdsa::Signature = signing_key.sign(message.as_bytes());
    let credential = Base64UrlUnpadded::encode_string(&sig.to_bytes());

    Json(json!({
        "credential": credential,
        "sign_message": message,
    }))
    .into_response()
}
