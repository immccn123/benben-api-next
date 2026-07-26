use base64ct::{Base64UrlUnpadded, Encoding};
use p256::SecretKey as P256SecretKey;
use p256::ecdsa::{
    Signature as P256Signature, SigningKey as P256SigningKey, VerifyingKey as P256VerifyingKey,
};
use signature::{Signer, Verifier};
use sm2::dsa::SigningKey as Sm2SigningKey;
use sm2::dsa::VerifyingKey as Sm2VerifyingKey;
use sm2::{PublicKey as Sm2PublicKey, SecretKey as Sm2SecretKey};

pub use crate::crypto::keys::KeyType;

pub fn p256_sign(private_key_pem: &str, message: &[u8]) -> Option<String> {
    let signing_key: P256SigningKey =
        pkcs8::DecodePrivateKey::from_pkcs8_pem(private_key_pem).ok()?;
    let sig: P256Signature = signing_key.sign(message);
    Some(Base64UrlUnpadded::encode_string(&sig.to_bytes()))
}

pub fn p256_verify(public_key_pem: &str, message: &[u8], signature_b64: &str) -> bool {
    let Some(verifying_key) = (|| {
        let vk: P256VerifyingKey =
            pkcs8::DecodePublicKey::from_public_key_pem(public_key_pem).ok()?;
        let sig_bytes = Base64UrlUnpadded::decode_vec(signature_b64).ok()?;
        let sig = P256Signature::from_slice(&sig_bytes).ok()?;
        Some((vk, sig))
    })() else {
        return false;
    };
    verifying_key.0.verify(message, &verifying_key.1).is_ok()
}

pub fn p256_sign_jwk(private_key_jwk: &str, message: &[u8]) -> Option<String> {
    let signing_key = p256_import_private_jwk(private_key_jwk)?;
    let sig: P256Signature = signing_key.sign(message);
    Some(Base64UrlUnpadded::encode_string(&sig.to_bytes()))
}

pub fn p256_verify_jwk(public_key_jwk: &str, message: &[u8], signature_b64: &str) -> bool {
    let Some((verifying_key, sig)) = (|| {
        let vk = p256_import_public_jwk(public_key_jwk)?;
        let sig_bytes = Base64UrlUnpadded::decode_vec(signature_b64).ok()?;
        let sig = P256Signature::from_slice(&sig_bytes).ok()?;
        Some((vk, sig))
    })() else {
        return false;
    };
    verifying_key.verify(message, &sig).is_ok()
}

fn p256_import_private_jwk(jwk: &str) -> Option<P256SigningKey> {
    let v: serde_json::Value = serde_json::from_str(jwk).ok()?;
    let d = v["d"].as_str()?;
    let d_bytes = Base64UrlUnpadded::decode_vec(d).ok()?;
    let secret = P256SecretKey::from_slice(&d_bytes).ok()?;
    Some(P256SigningKey::from(secret))
}

fn p256_import_public_jwk(jwk: &str) -> Option<P256VerifyingKey> {
    let v: serde_json::Value = serde_json::from_str(jwk).ok()?;
    let x = v["x"].as_str()?;
    let y = v["y"].as_str()?;
    let x_bytes = Base64UrlUnpadded::decode_vec(x).ok()?;
    let y_bytes = Base64UrlUnpadded::decode_vec(y).ok()?;
    let mut point_bytes = vec![0x04];
    point_bytes.extend_from_slice(&x_bytes);
    point_bytes.extend_from_slice(&y_bytes);
    P256VerifyingKey::from_sec1_bytes(&point_bytes).ok()
}

pub fn sm2_sign(distid: &str, private_key_pem: &str, message: &[u8]) -> Option<String> {
    let secret_key: Sm2SecretKey = pkcs8::DecodePrivateKey::from_pkcs8_pem(private_key_pem).ok()?;
    let signing_key = Sm2SigningKey::new(distid, &secret_key).ok()?;
    let sig: sm2::dsa::Signature = signing_key.sign(message);
    Some(Base64UrlUnpadded::encode_string(&sig.to_bytes()))
}

pub fn sm2_verify(distid: &str, public_key_pem: &str, message: &[u8], signature_b64: &str) -> bool {
    let Some((verifying_key, sig)) = (|| {
        let public_key: Sm2PublicKey =
            pkcs8::DecodePublicKey::from_public_key_pem(public_key_pem).ok()?;
        let vk = Sm2VerifyingKey::new(distid, public_key).ok()?;
        let sig_bytes = Base64UrlUnpadded::decode_vec(signature_b64).ok()?;
        let sig = sm2::dsa::Signature::from_slice(&sig_bytes).ok()?;
        Some((vk, sig))
    })() else {
        return false;
    };
    verifying_key.verify(message, &sig).is_ok()
}

pub fn verify(
    key_type: KeyType,
    distid: Option<&str>,
    public_key_pem: &str,
    message: &[u8],
    signature_b64: &str,
) -> bool {
    match key_type {
        KeyType::P256 => p256_verify(public_key_pem, message, signature_b64),
        KeyType::Sm2 => sm2_verify(distid.unwrap_or(""), public_key_pem, message, signature_b64),
    }
}

pub fn sign(
    key_type: KeyType,
    distid: Option<&str>,
    private_key_pem: &str,
    message: &[u8],
) -> Option<String> {
    match key_type {
        KeyType::P256 => p256_sign(private_key_pem, message),
        KeyType::Sm2 => sm2_sign(distid.unwrap_or(""), private_key_pem, message),
    }
}

pub fn detect_key_type_from_pem(pem: &str) -> KeyType {
    if pem.contains("SM2") || pem.contains("1.2.156.10197.1.301") {
        KeyType::Sm2
    } else {
        KeyType::P256
    }
}

pub fn p256_verify_raw(public_key_sec1: &[u8], message: &[u8], signature_b64: &str) -> bool {
    let Some((vk, sig)) = (|| {
        let vk = P256VerifyingKey::from_sec1_bytes(public_key_sec1).ok()?;
        let sig_bytes = Base64UrlUnpadded::decode_vec(signature_b64).ok()?;
        let sig = P256Signature::from_slice(&sig_bytes).ok()?;
        Some((vk, sig))
    })() else {
        return false;
    };
    vk.verify(message, &sig).is_ok()
}

pub fn sm2_verify_raw(
    distid: &str,
    public_key_sec1: &[u8],
    message: &[u8],
    signature_b64: &str,
) -> bool {
    let Some((vk, sig)) = (|| {
        let pk = Sm2PublicKey::from_sec1_bytes(public_key_sec1).ok()?;
        let vk = Sm2VerifyingKey::new(distid, pk).ok()?;
        let sig_bytes = Base64UrlUnpadded::decode_vec(signature_b64).ok()?;
        let sig = sm2::dsa::Signature::from_slice(&sig_bytes).ok()?;
        Some((vk, sig))
    })() else {
        return false;
    };
    vk.verify(message, &sig).is_ok()
}

pub fn verify_raw(
    key_type: KeyType,
    distid: Option<&str>,
    public_key_sec1: &[u8],
    message: &[u8],
    signature_b64: &str,
) -> bool {
    match key_type {
        KeyType::P256 => p256_verify_raw(public_key_sec1, message, signature_b64),
        KeyType::Sm2 => sm2_verify_raw(
            distid.unwrap_or(""),
            public_key_sec1,
            message,
            signature_b64,
        ),
    }
}
