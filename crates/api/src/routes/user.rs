use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use chrono::{Datelike, Timelike};
// use ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey, VerifyingKey};
// use pkcs8::{DecodePrivateKey, DecodePublicKey};
use ed25519_dalek::Signature;

use crate::{
    err::AppError,
    middleware::{counter::HitCountGuard, env_exposer::EnvGuard},
    utils::sekai::*,
};

#[get("/anon/create")]
async fn create_anon_user_id(
    count: HitCountGuard<'_>,
    env: EnvGuard<'_>,
) -> Result<String, AppError> {
    let count = count.increment();
    let days = chrono::Utc::now().num_days_from_ce();
    let minutes_today = chrono::Utc::now().num_seconds_from_midnight() / 60;

    let pre32 = ((days as u32) << 16) | minutes_today;
    let pre32 = half_sekai_transform(pre32).to_le_bytes();
    let suf64 = sekai_transform(count).to_le_bytes();

    let mut buffer = Vec::with_capacity(pre32.len() + suf64.len());
    buffer.extend_from_slice(&pre32);
    buffer.extend_from_slice(&suf64);

    let _ = buffer;
    let _ = env;

    // let mut private_key = SigningKey::from_pkcs8_pem(
    //     format!(
    //         "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
    //         env.get("ANON_PRIVATE_KEY").unwrap()
    //     )
    //     .as_str(),
    // )
    // .map_err(|_| AppError::TypeErr("Private Key Decode Failed"))?;

    // let signature = private_key.sign(buffer.as_slice());
    // let signature_base64 = BASE64_URL_SAFE.encode(signature.to_bytes());
    // let msg_base64 = BASE64_URL_SAFE.encode(buffer.as_slice());

    // Ok(format!("{msg_base64}@{signature_base64}"))
    Err(AppError::UnimplementedYet)
}

#[get("/anon/valide?<code>")]
async fn valide_anon_user_id(code: String, env: EnvGuard<'_>) -> Result<String, AppError> {
    let parts: Vec<&str> = code.split('@').collect();
    if parts.len() != 2 {
        return Err(AppError::TypeErr("Invalid ID format"));
    }
    let msg_base64 = parts[0];
    let sig_base64 = parts[1];

    let msg_buffer = BASE64_URL_SAFE
        .decode(msg_base64)
        .map_err(|_| AppError::TypeErr("Base64 Decode Failed for Message"))?;
    let sig_bytes = BASE64_URL_SAFE
        .decode(sig_base64)
        .map_err(|_| AppError::TypeErr("Base64 Decode Failed for Signature"))?;

    if sig_bytes.len() != 64 {
        return Err(AppError::TypeErr("Signature Length Invalid"));
    }

    let signature = Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());

    let _ = signature;
    let _ = msg_buffer;
    let _ = env;

    // let public_key = VerifyingKey::from_public_key_pem(
    //     format!(
    //         "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
    //         env.get("ANON_PUBLIC_KEY").unwrap_or_default()
    //     )
    //     .as_str(),
    // )
    // .map_err(|_| AppError::TypeErr("Public Key Decode Failed"))?;

    // public_key
    //     .verify_strict(msg_buffer.as_slice(), &signature)
    //     .map_err(|_| AppError::TypeErr("Signature Verification Failed"))?;

    // if msg_buffer.len() != 12 {
    //     return Err(AppError::TypeErr("Message buffer length invalid"));
    // }

    // let user_id: u128 = {
    //     let mut id_bytes = [0u8; 16];
    //     id_bytes[0..12].copy_from_slice(msg_buffer.as_slice());

    //     u128::from_le_bytes(id_bytes)
    // };

    // Ok(format!("Verification Successful. User Id: {user_id}"))
    Err(AppError::UnimplementedYet)
}

export_routes![create_anon_user_id, valide_anon_user_id];
