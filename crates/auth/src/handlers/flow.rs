use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Json, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::Utc;
use nanoid::nanoid;
use p256::ecdsa::Signature as P256Signature;
use p256::ecdsa::signature::Verifier;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use url::form_urlencoded;

use crate::AppState;
use crate::crypto::derive;
use crate::crypto::jwt;

#[derive(Deserialize)]
pub struct AuthQuery {
    pub app_id: String,
    pub callback_url: String,
    pub credential: String,
    pub state: Option<String>,
}

pub async fn auth(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<AuthQuery>,
) -> Response {
    if let Err(e) = verify_credential(&state, &q.app_id, &q.callback_url, &q.credential) {
        return error_page(&state, &e);
    }

    let nonce = nanoid!();
    let jar = jar
        .add(sso_cookie("sso_nonce", &nonce))
        .add(sso_cookie("sso_callback", &q.callback_url))
        .add(sso_cookie("sso_appstate", &q.state.unwrap_or_default()))
        .add(sso_cookie("sso_app_id", &q.app_id));

    let cpo_url = format!(
        "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid profile&state={}",
        state.config.cpoauth_base,
        form_urlencoded::byte_serialize(state.config.cpoauth_client_id.as_bytes())
            .collect::<String>(),
        form_urlencoded::byte_serialize(state.config.cpoauth_redirect_uri.as_bytes())
            .collect::<String>(),
        form_urlencoded::byte_serialize(nonce.as_bytes()).collect::<String>(),
    );

    let mut ctx = tera::Context::new();
    ctx.insert("cpo_url", &cpo_url);
    let html = state
        .tera
        .render("confirm.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Error</h1><pre>{e}</pre>"));
    (jar, Html(html)).into_response()
}

pub async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let state_param = params.get("state").cloned().unwrap_or_default();
    let nonce = jar
        .get("sso_nonce")
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    if nonce.is_empty() || nonce != state_param {
        return error_page(&state, "invalid state");
    }

    let app_id = jar
        .get("sso_app_id")
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    if app_id.is_empty() {
        return error_page(&state, "missing app_id");
    }

    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => return error_page(&state, "CPOAuth returned no code"),
    };

    let tokens = match exchange_cpoauth(&state, &code).await {
        Ok(t) => t,
        Err(e) => return error_page(&state, &e),
    };

    let access_token = tokens["access_token"].as_str().unwrap_or("");
    let user = match fetch_userinfo(&state, access_token).await {
        Ok(u) => u,
        Err(e) => return error_page(&state, &e),
    };

    let now = Utc::now().timestamp() as usize;
    let mut claims = serde_json::Map::new();
    for (k, v) in &user {
        claims.insert(k.clone(), v.clone());
    }
    claims.insert("iss".into(), json!(state.config.base_url));
    claims.insert("aud".into(), json!(app_id));
    claims.insert("jti".into(), json!(nanoid!()));
    claims.insert("iat".into(), json!(now));
    claims.insert("exp".into(), json!(now + 604800));
    claims.insert("refreshes_left".into(), json!(12));

    let jwt = jwt::create_jwt(
        &serde_json::Value::Object(claims),
        &state.config.jwt_sign_key,
    )
    .unwrap_or_else(|_| "jwt_error".to_string());

    let callback_url = jar
        .get("sso_callback")
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    let app_state = jar
        .get("sso_appstate")
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    let username = user["username"].as_str().unwrap_or("").to_string();

    let jar = jar
        .remove(sso_cookie("sso_nonce", ""))
        .remove(sso_cookie("sso_app_id", ""));

    let mut ctx = tera::Context::new();
    ctx.insert("username", &username);
    ctx.insert("jwt", &jwt);
    ctx.insert("callback_url", &callback_url);
    ctx.insert("state", &app_state);
    let html = state
        .tera
        .render("complete.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Error</h1><pre>{e}</pre>"));
    (jar, Html(html)).into_response()
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub token: String,
    pub app_id: String,
    pub signature: String,
    pub timestamp: u64,
}

pub async fn refresh(State(state): State<AppState>, Json(body): Json<RefreshRequest>) -> Response {
    let now = Utc::now().timestamp() as u64;
    if body.timestamp > now + 30 || body.timestamp < now.saturating_sub(30) {
        return Json(json!({"error": "timestamp expired"})).into_response();
    }

    let root_pub = match derive::root_pub_from_pem(&state.config.client_verify_key) {
        Some(k) => k,
        None => return Json(json!({"error": "server key misconfigured"})).into_response(),
    };

    let app_vk = derive::derive_verifying_key(&root_pub, &body.app_id);

    let sign_payload = format!("{}{}", body.token, body.timestamp);
    let sig_bytes = match Base64UrlUnpadded::decode_vec(&body.signature) {
        Ok(b) => b,
        Err(_) => return Json(json!({"error": "invalid signature encoding"})).into_response(),
    };
    let sig = match P256Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return Json(json!({"error": "invalid signature format"})).into_response(),
    };
    if app_vk.verify(sign_payload.as_bytes(), &sig).is_err() {
        return Json(json!({"error": "client signature invalid"})).into_response();
    }

    let mut claims = match jwt::verify_jwt(&body.token, &state.config.jwt_verify_key) {
        Ok(c) => c,
        Err(_) => return Json(json!({"error": "invalid token"})).into_response(),
    };

    if claims["aud"].as_str().unwrap_or("") != body.app_id {
        return Json(json!({"error": "token not issued for this app"})).into_response();
    }

    let left = claims["refreshes_left"].as_i64().unwrap_or(0);
    if left <= 0 {
        return Json(json!({"error": "no refreshes left"})).into_response();
    }
    let now_ts = Utc::now().timestamp() as usize;
    if let Some(obj) = claims.as_object_mut() {
        obj.insert("iat".into(), json!(now_ts));
        obj.insert("exp".into(), json!(now_ts + 604800));
        obj.insert("refreshes_left".into(), json!(left - 1));
    }
    let jwt = jwt::create_jwt(&claims, &state.config.jwt_sign_key)
        .unwrap_or_else(|_| "jwt_error".to_string());
    Json(json!({"token": jwt})).into_response()
}

fn verify_credential(
    state: &AppState,
    app_id: &str,
    callback_url: &str,
    credential: &str,
) -> Result<(), String> {
    let root_pub =
        derive::root_pub_from_pem(&state.config.client_verify_key).ok_or("服务端根公钥无效")?;

    let app_vk = derive::derive_verifying_key(&root_pub, app_id);

    let message = format!("ba+{app_id}+{callback_url}");
    let sig_bytes = Base64UrlUnpadded::decode_vec(credential).map_err(|_| "凭证解码失败")?;
    let sig = P256Signature::from_slice(&sig_bytes).map_err(|_| "凭证签名格式无效")?;

    app_vk
        .verify(message.as_bytes(), &sig)
        .map_err(|_| "凭证签名验证不通过".to_string())
}

async fn exchange_cpoauth(
    state: &AppState,
    code: &str,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/oauth/token", state.config.cpoauth_base))
        .header("Content-Type", "application/json")
        .json(&json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": state.config.cpoauth_redirect_uri,
            "client_id": state.config.cpoauth_client_id,
            "client_secret": state.config.cpoauth_client_secret,
        }))
        .send()
        .await
        .map_err(|e| format!("CPOAuth: {e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("token: {body}"));
    }
    resp.json().await.map_err(|e| format!("parse: {e}"))
}

async fn fetch_userinfo(
    state: &AppState,
    access_token: &str,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/oauth/userinfo", state.config.cpoauth_base))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("userinfo: {e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("userinfo: {body}"));
    }
    resp.json().await.map_err(|e| format!("parse: {e}"))
}

fn error_page(state: &AppState, msg: &str) -> Response {
    let mut ctx = tera::Context::new();
    ctx.insert("error", msg);
    let html = state
        .tera
        .render("error.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Error</h1><pre>{e}</pre>"));
    Html(html).into_response()
}

fn sso_cookie(name: &str, value: &str) -> Cookie<'static> {
    Cookie::build((name.to_string(), value.to_string()))
        .path("/")
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .http_only(true)
        .build()
}
