use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::SigningKey as P256SigningKey;
use p256::ecdsa::VerifyingKey as P256VerifyingKey;
use p256::elliptic_curve::sec1::ToSec1Point;
use p256::{PublicKey as P256PublicKey, SecretKey as P256SecretKey};
use sm2::dsa::SigningKey as Sm2DsaSigningKey;
use sm2::dsa::VerifyingKey as Sm2DsaVerifyingKey;
use sm2::elliptic_curve::Generate;
use sm2::{PublicKey as Sm2PublicKey, SecretKey as Sm2SecretKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    P256,
    Sm2,
}

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub key_type: KeyType,
    pub public_key_pem: String,
    pub private_key_pem: String,
    pub public_key_jwk: String,
    pub private_key_jwk: String,
    pub distid: Option<String>,
}

pub fn generate_p256_key_pair() -> KeyPair {
    let secret = P256SecretKey::generate();
    let signing_key = P256SigningKey::from(secret);
    let verifying_key = signing_key.verifying_key();

    let private_key_pem = {
        pkcs8::EncodePrivateKey::to_pkcs8_pem(&signing_key, pkcs8::LineEnding::LF)
            .unwrap()
            .to_string()
    };

    let public_key_pem = {
        let pub_key: P256PublicKey = verifying_key.clone().into();
        pkcs8::EncodePublicKey::to_public_key_pem(&pub_key, pkcs8::LineEnding::LF).unwrap()
    };

    let public_key_jwk = p256_jwk_from_verifying_key(&verifying_key);
    let private_key_jwk = p256_jwk_from_signing_key(&signing_key);

    KeyPair {
        key_type: KeyType::P256,
        public_key_pem,
        private_key_pem,
        public_key_jwk,
        private_key_jwk,
        distid: None,
    }
}

pub fn generate_sm2_key_pair(distid: &str) -> KeyPair {
    let secret = Sm2SecretKey::generate();
    let dsa_signing_key =
        Sm2DsaSigningKey::new(distid, &secret).expect("Failed to create SM2 signing key");
    let dsa_verifying_key = dsa_signing_key.verifying_key().clone();

    let public_key = Sm2PublicKey::from_secret_scalar(&secret.to_nonzero_scalar());

    let private_key_pem = {
        pkcs8::EncodePrivateKey::to_pkcs8_pem(&secret, pkcs8::LineEnding::LF)
            .unwrap()
            .to_string()
    };

    let public_key_pem =
        { pkcs8::EncodePublicKey::to_public_key_pem(&public_key, pkcs8::LineEnding::LF).unwrap() };

    let public_key_jwk = sm2_jwk_from_verifying_key(&dsa_verifying_key);
    let private_key_jwk = sm2_jwk_from_secret_key(&secret);

    KeyPair {
        key_type: KeyType::Sm2,
        public_key_pem,
        private_key_pem,
        public_key_jwk,
        private_key_jwk,
        distid: Some(distid.to_string()),
    }
}

fn p256_jwk_from_verifying_key(vk: &P256VerifyingKey) -> String {
    let point = vk.as_affine().to_sec1_point(false);
    let x = Base64UrlUnpadded::encode_string(point.x().unwrap());
    let y = Base64UrlUnpadded::encode_string(point.y().unwrap());
    format!(r#"{{"kty":"EC","crv":"P-256","x":"{x}","y":"{y}"}}"#)
}

fn p256_jwk_from_signing_key(sk: &P256SigningKey) -> String {
    let vk = sk.verifying_key();
    let point = vk.as_affine().to_sec1_point(false);
    let x = Base64UrlUnpadded::encode_string(point.x().unwrap());
    let y = Base64UrlUnpadded::encode_string(point.y().unwrap());
    let d_bytes = sk.as_nonzero_scalar().to_bytes();
    let d = Base64UrlUnpadded::encode_string(&d_bytes);
    format!(r#"{{"kty":"EC","crv":"P-256","x":"{x}","y":"{y}","d":"{d}"}}"#)
}

fn sm2_jwk_from_verifying_key(vk: &Sm2DsaVerifyingKey) -> String {
    let point = vk.as_affine().to_sec1_point(false);
    let x = Base64UrlUnpadded::encode_string(point.x().unwrap());
    let y = Base64UrlUnpadded::encode_string(point.y().unwrap());
    format!(r#"{{"kty":"EC","crv":"SM2","x":"{x}","y":"{y}"}}"#)
}

fn sm2_jwk_from_secret_key(sk: &Sm2SecretKey) -> String {
    let public_key = Sm2PublicKey::from_secret_scalar(&sk.to_nonzero_scalar());
    let point = public_key.as_affine().to_sec1_point(false);
    let x = Base64UrlUnpadded::encode_string(point.x().unwrap());
    let y = Base64UrlUnpadded::encode_string(point.y().unwrap());
    let d_bytes = sk.to_nonzero_scalar().to_bytes();
    let d = Base64UrlUnpadded::encode_string(&d_bytes);
    format!(r#"{{"kty":"EC","crv":"SM2","x":"{x}","y":"{y}","d":"{d}"}}"#)
}
